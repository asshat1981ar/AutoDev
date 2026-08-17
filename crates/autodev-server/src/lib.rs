mod events;
mod objective;
mod paths;
mod runner;
mod store;

use std::{convert::Infallible, sync::Arc};

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use forge_core::{TaskGraph, VerifiedOrchestratorState};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::Sha256;
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use uuid::Uuid;

pub use events::ObjectiveEvent;
pub use objective::{ObjectiveStatus, ObjectiveView};
pub use paths::{default_state_dir, validate_control_plane_paths, ControlPlanePathError};
pub use runner::{
    run_objective_cycle, run_objective_loop, ActionProposer, ModelActionProposer,
    ObjectiveApprovalGrant, ObjectiveRunner, RunnerError, RunnerExecution, VerificationFactory,
};
pub use store::{
    FileObjectiveStore, InMemoryObjectiveStore, ObjectiveSnapshot, ObjectiveStore, StoreError,
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectiveRequest {
    pub repository: String,
    pub description: String,
    #[serde(default)]
    pub branch: Option<String>,
}

pub struct AppState<S: ObjectiveStore = InMemoryObjectiveStore> {
    store: Arc<S>,
    events: broadcast::Sender<ObjectiveEvent>,
    github_webhook_secret: Option<String>,
}

impl<S: ObjectiveStore> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            events: self.events.clone(),
            github_webhook_secret: self.github_webhook_secret.clone(),
        }
    }
}

impl AppState<InMemoryObjectiveStore> {
    pub fn new(github_webhook_secret: Option<String>) -> Self {
        Self::with_store(
            github_webhook_secret,
            Arc::new(InMemoryObjectiveStore::default()),
        )
    }
}

impl<S: ObjectiveStore> AppState<S> {
    pub fn with_store(github_webhook_secret: Option<String>, store: Arc<S>) -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            store,
            events,
            github_webhook_secret: github_webhook_secret.filter(|value| !value.trim().is_empty()),
        }
    }

    pub fn store(&self) -> Arc<S> {
        self.store.clone()
    }

    pub fn event_sender(&self) -> broadcast::Sender<ObjectiveEvent> {
        self.events.clone()
    }

    fn enqueue(&self, request: ObjectiveRequest) -> Result<ObjectiveView, EnqueueError> {
        if request.repository.trim().is_empty() {
            return Err(EnqueueError::Validation("repository is required"));
        }
        if request.description.trim().is_empty() {
            return Err(EnqueueError::Validation("description is required"));
        }

        let id = Uuid::new_v4().to_string();
        let branch = request
            .branch
            .filter(|branch| !branch.trim().is_empty())
            .unwrap_or_else(|| format!("autodev/objective-{}", &id[..8]));
        let graph = TaskGraph::single(&format!("Objective {id}"), request.description.trim());
        let view = ObjectiveView {
            id: id.clone(),
            repository: request.repository.trim().to_string(),
            description: request.description.trim().to_string(),
            branch,
            status: ObjectiveStatus::Queued,
            current_task_id: Some(graph.root.clone()),
            current_phase: None,
            latest_evidence_ref: None,
            blocked_reason: None,
        };
        let snapshot = ObjectiveSnapshot {
            view: view.clone(),
            graph,
            orchestrator: VerifiedOrchestratorState::default(),
            evidence: vec![],
        };
        self.store.put(&snapshot)?;
        let _ = self
            .events
            .send(ObjectiveEvent::from_view(&view, "objective accepted"));
        Ok(view)
    }
}

pub fn router<S: ObjectiveStore>(state: AppState<S>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/api/v1/objectives",
            get(list_objectives::<S>).post(create_objective::<S>),
        )
        .route("/api/v1/objectives/:id", get(get_objective::<S>))
        .route("/api/v1/events/stream", get(event_stream::<S>))
        .route("/events", get(event_stream::<S>))
        .route("/webhooks/github", post(github_webhook::<S>))
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn list_objectives<S: ObjectiveStore>(State(state): State<AppState<S>>) -> Response {
    match state.store.load_all() {
        Ok(snapshots) => Json(
            snapshots
                .into_iter()
                .map(|snapshot| snapshot.view)
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(error) => store_error(error),
    }
}

async fn get_objective<S: ObjectiveStore>(
    State(state): State<AppState<S>>,
    Path(id): Path<String>,
) -> Response {
    match state.store.get(&id) {
        Ok(Some(snapshot)) => Json(snapshot.view).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "objective_not_found"})),
        )
            .into_response(),
        Err(StoreError::InvalidId(_)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_objective_id"})),
        )
            .into_response(),
        Err(error) => store_error(error),
    }
}

async fn create_objective<S: ObjectiveStore>(
    State(state): State<AppState<S>>,
    Json(request): Json<ObjectiveRequest>,
) -> Response {
    match state.enqueue(request) {
        Ok(view) => (StatusCode::ACCEPTED, Json(view)).into_response(),
        Err(EnqueueError::Validation(message)) => {
            (StatusCode::BAD_REQUEST, Json(json!({"error": message}))).into_response()
        }
        Err(EnqueueError::Store(error)) => store_error(error),
    }
}

async fn event_stream<S: ObjectiveStore>(
    State(state): State<AppState<S>>,
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

async fn github_webhook<S: ObjectiveStore>(
    State(state): State<AppState<S>>,
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
    match state.enqueue(request) {
        Ok(view) => (StatusCode::ACCEPTED, Json(view)).into_response(),
        Err(EnqueueError::Validation(message)) => {
            (StatusCode::BAD_REQUEST, Json(json!({"error": message}))).into_response()
        }
        Err(EnqueueError::Store(error)) => store_error(error),
    }
}

fn store_error(error: StoreError) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": "objective_store_error", "detail": error.to_string()})),
    )
        .into_response()
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

#[derive(Debug, thiserror::Error)]
enum EnqueueError {
    #[error("{0}")]
    Validation(&'static str),
    #[error(transparent)]
    Store(#[from] StoreError),
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
    async fn objective_intake_creates_internal_forgecore_task_graph() {
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

        let snapshots = state.store.load_all().expect("stored objectives");
        let snapshot = snapshots.first().expect("queued objective");
        assert_eq!(
            snapshot.graph.root().description,
            "Implement health endpoint"
        );
        assert_eq!(snapshot.view.status, ObjectiveStatus::Queued);
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
