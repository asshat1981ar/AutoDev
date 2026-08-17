use std::{collections::BTreeMap, convert::Infallible, sync::Arc};

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use forge_core::TaskGraph;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectiveRequest {
    pub repository: String,
    pub description: String,
    #[serde(default)]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectiveRecord {
    pub id: String,
    pub repository: String,
    pub description: String,
    pub branch: String,
    pub status: String,
    pub graph: TaskGraph,
}

#[derive(Clone)]
pub struct AppState {
    objectives: Arc<RwLock<BTreeMap<String, ObjectiveRecord>>>,
    events: broadcast::Sender<String>,
    github_webhook_secret: Option<String>,
}

impl AppState {
    pub fn new(github_webhook_secret: Option<String>) -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            objectives: Arc::new(RwLock::new(BTreeMap::new())),
            events,
            github_webhook_secret: github_webhook_secret.filter(|value| !value.trim().is_empty()),
        }
    }

    async fn enqueue(&self, request: ObjectiveRequest) -> Result<ObjectiveRecord, &'static str> {
        if request.repository.trim().is_empty() {
            return Err("repository is required");
        }
        if request.description.trim().is_empty() {
            return Err("description is required");
        }

        let id = Uuid::new_v4().to_string();
        let branch = request
            .branch
            .filter(|branch| !branch.trim().is_empty())
            .unwrap_or_else(|| format!("autodev/objective-{}", &id[..8]));
        let graph = TaskGraph::single(&format!("Objective {id}"), request.description.trim());
        let record = ObjectiveRecord {
            id: id.clone(),
            repository: request.repository.trim().to_string(),
            description: request.description.trim().to_string(),
            branch,
            status: "queued".to_string(),
            graph,
        };

        self.objectives
            .write()
            .await
            .insert(id.clone(), record.clone());
        let _ = self.events.send(
            json!({
                "type": "objective_queued",
                "data": {
                    "objective_id": id,
                    "repository": record.repository,
                    "branch": record.branch,
                    "status": record.status,
                }
            })
            .to_string(),
        );
        Ok(record)
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/api/v1/objectives",
            get(list_objectives).post(create_objective),
        )
        .route("/api/v1/events/stream", get(event_stream))
        .route("/events", get(event_stream))
        .route("/webhooks/github", post(github_webhook))
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn list_objectives(State(state): State<AppState>) -> Json<Vec<ObjectiveRecord>> {
    let objectives = state.objectives.read().await;
    Json(objectives.values().cloned().collect())
}

async fn create_objective(
    State(state): State<AppState>,
    Json(request): Json<ObjectiveRequest>,
) -> Response {
    match state.enqueue(request).await {
        Ok(record) => (StatusCode::ACCEPTED, Json(record)).into_response(),
        Err(message) => (StatusCode::BAD_REQUEST, Json(json!({"error": message}))).into_response(),
    }
}

async fn event_stream(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let stream =
        BroadcastStream::new(state.events.subscribe()).filter_map(|message| match message {
            Ok(data) => Some(Ok(Event::default().data(data))),
            Err(_) => None,
        });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn github_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(secret) = state.github_webhook_secret.as_deref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "GITHUB_WEBHOOK_SECRET is not configured"})),
        )
            .into_response();
    };

    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|value| value.to_str().ok());
    if !verify_github_signature(secret.as_bytes(), signature, &body) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid signature"})),
        )
            .into_response();
    }

    let payload: Value = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid JSON"})),
            )
                .into_response()
        }
    };

    let event_type = headers
        .get("x-github-event")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown");
    if event_type != "issues" || payload.get("action").and_then(Value::as_str) != Some("opened") {
        return (StatusCode::OK, Json(json!({"status": "ignored"}))).into_response();
    }

    let issue_number = payload
        .pointer("/issue/number")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let title = payload
        .pointer("/issue/title")
        .and_then(Value::as_str)
        .unwrap_or("Untitled issue");
    let repository = payload
        .pointer("/repository/full_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if issue_number == 0 || repository.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "issue number and repository are required"})),
        )
            .into_response();
    }

    let request = ObjectiveRequest {
        repository: repository.to_string(),
        description: format!("Issue #{issue_number}: {title}"),
        branch: Some(format!("autodev/issue-{issue_number}")),
    };
    match state.enqueue(request).await {
        Ok(record) => (StatusCode::ACCEPTED, Json(record)).into_response(),
        Err(message) => (StatusCode::BAD_REQUEST, Json(json!({"error": message}))).into_response(),
    }
}

fn verify_github_signature(secret: &[u8], signature_header: Option<&str>, body: &[u8]) -> bool {
    let Some(signature_hex) = signature_header.and_then(|value| value.strip_prefix("sha256="))
    else {
        return false;
    };
    let Ok(signature) = hex::decode(signature_hex) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(secret) else {
        return false;
    };
    mac.update(body);
    mac.verify_slice(&signature).is_ok()
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use super::*;

    fn signed(secret: &[u8], body: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret).expect("valid HMAC key");
        mac.update(body);
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    }

    #[test]
    fn github_signature_verification_is_fail_closed() {
        let body = br#"{"action":"opened"}"#;
        let signature = signed(b"secret", body);
        assert!(verify_github_signature(b"secret", Some(&signature), body));
        assert!(!verify_github_signature(b"wrong", Some(&signature), body));
        assert!(!verify_github_signature(b"secret", None, body));
    }

    #[tokio::test]
    async fn objective_intake_creates_a_forgecore_task_graph() {
        let state = AppState::new(Some("secret".to_string()));
        let app = router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/objectives")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"repository":"owner/repo","description":"Implement health endpoint"}"#,
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let objectives = state.objectives.read().await;
        let record = objectives.values().next().expect("queued objective");
        assert_eq!(record.graph.root().description, "Implement health endpoint");
        assert_eq!(record.status, "queued");
    }

    #[tokio::test]
    async fn unsigned_github_webhook_is_rejected() {
        let app = router(AppState::new(Some("secret".to_string())));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/github")
                    .header("x-github-event", "issues")
                    .body(Body::from("{}"))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
