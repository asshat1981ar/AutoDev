use std::{collections::BTreeMap, convert::Infallible, sync::Arc};

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::{self, Next},
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

mod mcp;
pub mod public_protocol;

use public_protocol::{PublicObjectiveCreate, PublicObjectiveEvent, PublicObjectiveSummary};

type HmacSha256 = Hmac<Sha256>;
const MCP_BEARER_COMPARE_KEY: &[u8] = b"autodev-mcp-bearer-constant-time-compare-v1";

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
    events: broadcast::Sender<PublicObjectiveEvent>,
    github_webhook_secret: Option<String>,
    mcp_bearer_tag: Option<Arc<Vec<u8>>>,
}

impl AppState {
    pub fn new(github_webhook_secret: Option<String>) -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            objectives: Arc::new(RwLock::new(BTreeMap::new())),
            events,
            github_webhook_secret: github_webhook_secret.filter(|value| !value.trim().is_empty()),
            mcp_bearer_tag: None,
        }
    }

    /// Configure the bearer token required for the public MCP route.
    ///
    /// The state stores only a deterministic HMAC tag of the configured token.
    /// An empty token keeps MCP authentication unconfigured, which makes the
    /// `/mcp` route fail closed with `503 Service Unavailable`.
    pub fn with_mcp_bearer_token(mut self, token: impl Into<String>) -> Self {
        let token = token.into();
        self.mcp_bearer_tag = if token.trim().is_empty() {
            None
        } else {
            Some(Arc::new(mcp_bearer_tag(&token)))
        };
        self
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
        let _ = self.events.send(PublicObjectiveEvent::queued(&record));
        Ok(record)
    }
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route(
            "/api/v1/objectives",
            get(list_objectives).post(create_objective),
        )
        .route("/api/v1/events/stream", get(event_stream))
        .route("/events", get(event_stream))
        .route("/webhooks/github", post(github_webhook))
        .with_state(state.clone());

    let mcp = Router::new()
        .nest_service("/mcp", mcp::service(state.clone()))
        .layer(DefaultBodyLimit::max(mcp::MCP_MAX_BODY_BYTES))
        .layer(middleware::from_fn_with_state(state, require_mcp_bearer));

    api.merge(mcp)
}

async fn require_mcp_bearer(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let Some(expected_tag) = state.mcp_bearer_tag.as_deref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "AUTODEV_MCP_BEARER_TOKEN is not configured"})),
        )
            .into_response();
    };

    let presented = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty());

    if !presented.is_some_and(|token| verify_mcp_bearer(expected_tag, token)) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid MCP bearer token"})),
        )
            .into_response();
    }

    next.run(request).await
}

fn mcp_bearer_tag(token: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(MCP_BEARER_COMPARE_KEY)
        .expect("constant MCP bearer comparison key is valid");
    mac.update(token.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn verify_mcp_bearer(expected_tag: &[u8], presented: &str) -> bool {
    let Ok(mut mac) = HmacSha256::new_from_slice(MCP_BEARER_COMPARE_KEY) else {
        return false;
    };
    mac.update(presented.as_bytes());
    mac.verify_slice(expected_tag).is_ok()
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn list_objectives(State(state): State<AppState>) -> Json<Vec<PublicObjectiveSummary>> {
    let objectives = state.objectives.read().await;
    Json(
        objectives
            .values()
            .map(PublicObjectiveSummary::from)
            .collect(),
    )
}

async fn create_objective(
    State(state): State<AppState>,
    Json(request): Json<PublicObjectiveCreate>,
) -> Response {
    let request = ObjectiveRequest {
        repository: request.repository,
        description: request.description,
        branch: request.branch,
    };
    match state.enqueue(request).await {
        Ok(record) => (
            StatusCode::ACCEPTED,
            Json(PublicObjectiveSummary::from(&record)),
        )
            .into_response(),
        Err(message) => (StatusCode::BAD_REQUEST, Json(json!({"error": message}))).into_response(),
    }
}

async fn event_stream(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let stream =
        BroadcastStream::new(state.events.subscribe()).filter_map(|message| match message {
            Ok(event) => serde_json::to_string(&event)
                .ok()
                .map(|data| Ok(Event::default().data(data))),
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

    #[test]
    fn mcp_bearer_verification_is_constant_time_over_tags() {
        let expected = mcp_bearer_tag("secret-token");
        assert!(verify_mcp_bearer(&expected, "secret-token"));
        assert!(!verify_mcp_bearer(&expected, "wrong-token"));
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
