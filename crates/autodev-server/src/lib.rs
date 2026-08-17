mod objective;
mod store;

pub use objective::{ObjectiveEvent, ObjectiveStatus, ObjectiveView};
pub use store::{FileObjectiveStore, ObjectiveSnapshot, ObjectiveStore, StoreError};

use std::{
    collections::BTreeMap,
    convert::Infallible,
    ffi::OsStr,
    sync::{Arc, Mutex as StdMutex},
};

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
use forge_core::{
    CodexAccount, CodexLoginStart, CodexRateLimits, CodexSubscriptionClient,
    CodexSubscriptionError, StdioCodexTransport, TaskGraph,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::Sha256;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub trait CodexAccountService: Send + Sync {
    fn account(&self) -> Result<CodexAccount, CodexSubscriptionError>;
    fn start_browser_login(&self) -> Result<CodexLoginStart, CodexSubscriptionError>;
    fn start_device_code_login(&self) -> Result<CodexLoginStart, CodexSubscriptionError>;
    fn rate_limits(&self) -> Result<CodexRateLimits, CodexSubscriptionError>;
    fn logout(&self) -> Result<(), CodexSubscriptionError>;
}

pub struct CodexProcessService {
    client: StdMutex<CodexSubscriptionClient<StdioCodexTransport>>,
}

impl CodexProcessService {
    pub fn spawn(
        binary: impl AsRef<OsStr>,
        client_version: &str,
    ) -> Result<Self, CodexSubscriptionError> {
        let transport = StdioCodexTransport::spawn(binary)?;
        let mut client = CodexSubscriptionClient::new(transport);
        client.initialize(client_version)?;
        Ok(Self {
            client: StdMutex::new(client),
        })
    }

    fn with_client<R>(
        &self,
        operation: impl FnOnce(
            &mut CodexSubscriptionClient<StdioCodexTransport>,
        ) -> Result<R, CodexSubscriptionError>,
    ) -> Result<R, CodexSubscriptionError> {
        let mut client = self.client.lock().map_err(|_| {
            CodexSubscriptionError::Protocol("Codex app-server client lock is poisoned".into())
        })?;
        operation(&mut client)
    }
}

impl CodexAccountService for CodexProcessService {
    fn account(&self) -> Result<CodexAccount, CodexSubscriptionError> {
        self.with_client(CodexSubscriptionClient::account)
    }

    fn start_browser_login(&self) -> Result<CodexLoginStart, CodexSubscriptionError> {
        self.with_client(CodexSubscriptionClient::start_browser_login)
    }

    fn start_device_code_login(&self) -> Result<CodexLoginStart, CodexSubscriptionError> {
        self.with_client(CodexSubscriptionClient::start_device_code_login)
    }

    fn rate_limits(&self) -> Result<CodexRateLimits, CodexSubscriptionError> {
        self.with_client(CodexSubscriptionClient::rate_limits)
    }

    fn logout(&self) -> Result<(), CodexSubscriptionError> {
        self.with_client(CodexSubscriptionClient::logout)
    }
}

struct UnavailableCodexService {
    reason: String,
}

impl UnavailableCodexService {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    fn unavailable<T>(&self) -> Result<T, CodexSubscriptionError> {
        Err(CodexSubscriptionError::ProviderUnavailable(
            self.reason.clone(),
        ))
    }
}

impl CodexAccountService for UnavailableCodexService {
    fn account(&self) -> Result<CodexAccount, CodexSubscriptionError> {
        self.unavailable()
    }

    fn start_browser_login(&self) -> Result<CodexLoginStart, CodexSubscriptionError> {
        self.unavailable()
    }

    fn start_device_code_login(&self) -> Result<CodexLoginStart, CodexSubscriptionError> {
        self.unavailable()
    }

    fn rate_limits(&self) -> Result<CodexRateLimits, CodexSubscriptionError> {
        self.unavailable()
    }

    fn logout(&self) -> Result<(), CodexSubscriptionError> {
        self.unavailable()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectiveRequest {
    pub repository: String,
    pub description: String,
    #[serde(default)]
    pub branch: Option<String>,
}

#[derive(Debug, Clone)]
struct ObjectiveRecord {
    view: ObjectiveView,
    graph: TaskGraph,
}

#[derive(Clone)]
pub struct AppState {
    objectives: Arc<RwLock<BTreeMap<String, ObjectiveRecord>>>,
    events: broadcast::Sender<String>,
    github_webhook_secret: Option<String>,
    codex: Arc<dyn CodexAccountService>,
}

impl AppState {
    pub fn new(github_webhook_secret: Option<String>) -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            objectives: Arc::new(RwLock::new(BTreeMap::new())),
            events,
            github_webhook_secret: github_webhook_secret.filter(|value| !value.trim().is_empty()),
            codex: Arc::new(UnavailableCodexService::new(
                "Codex provider is not configured",
            )),
        }
    }

    pub fn with_codex_service(
        github_webhook_secret: Option<String>,
        codex: Arc<dyn CodexAccountService>,
    ) -> Self {
        let mut state = Self::new(github_webhook_secret);
        state.codex = codex;
        state
    }

    async fn enqueue(&self, request: ObjectiveRequest) -> Result<ObjectiveView, &'static str> {
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
        let mut record = ObjectiveRecord {
            view: ObjectiveView {
                id: id.clone(),
                repository: request.repository.trim().to_string(),
                description: request.description.trim().to_string(),
                branch,
                status: ObjectiveStatus::Queued,
                current_task_id: None,
                current_phase: None,
                latest_evidence_ref: None,
                blocked_reason: None,
            },
            graph,
        };
        record.view.current_task_id = Some(record.graph.root.clone());
        let view = record.view.clone();

        self.objectives.write().await.insert(id, record);
        if let Ok(payload) =
            serde_json::to_string(&ObjectiveEvent::from_view(&view, "objective accepted"))
        {
            let _ = self.events.send(payload);
        }
        Ok(view)
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/api/v1/objectives",
            get(list_objectives).post(create_objective),
        )
        .route("/api/v1/objectives/:id", get(get_objective))
        .route("/api/v1/events/stream", get(event_stream))
        .route("/api/v1/codex/account", get(codex_account))
        .route("/api/v1/codex/login/browser", post(codex_browser_login))
        .route(
            "/api/v1/codex/login/device-code",
            post(codex_device_code_login),
        )
        .route("/api/v1/codex/rate-limits", get(codex_rate_limits))
        .route("/api/v1/codex/logout", post(codex_logout))
        .route("/events", get(event_stream))
        .route("/webhooks/github", post(github_webhook))
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn list_objectives(State(state): State<AppState>) -> Json<Vec<ObjectiveView>> {
    let objectives = state.objectives.read().await;
    Json(
        objectives
            .values()
            .map(|record| record.view.clone())
            .collect(),
    )
}

async fn get_objective(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let objectives = state.objectives.read().await;
    match objectives.get(&id) {
        Some(record) => Json(record.view.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "objective_not_found"})),
        )
            .into_response(),
    }
}

async fn create_objective(
    State(state): State<AppState>,
    Json(request): Json<ObjectiveRequest>,
) -> Response {
    match state.enqueue(request).await {
        Ok(view) => (StatusCode::ACCEPTED, Json(view)).into_response(),
        Err(message) => (StatusCode::BAD_REQUEST, Json(json!({"error": message}))).into_response(),
    }
}

async fn codex_account(State(state): State<AppState>) -> Response {
    match run_codex(state.codex, |service| service.account()).await {
        Ok(account) => Json(account).into_response(),
        Err(response) => response,
    }
}

async fn codex_browser_login(State(state): State<AppState>) -> Response {
    match run_codex(state.codex, |service| service.start_browser_login()).await {
        Ok(CodexLoginStart::Browser { login_id, auth_url }) => Json(json!({
            "type": "browser",
            "login_id": login_id,
            "auth_url": auth_url,
        }))
        .into_response(),
        Ok(CodexLoginStart::DeviceCode { .. }) => codex_error_response(
            CodexSubscriptionError::Protocol("unexpected device-code login response".into()),
        ),
        Err(response) => response,
    }
}

async fn codex_device_code_login(State(state): State<AppState>) -> Response {
    match run_codex(state.codex, |service| service.start_device_code_login()).await {
        Ok(CodexLoginStart::DeviceCode {
            login_id,
            verification_url,
            user_code,
        }) => Json(json!({
            "type": "device_code",
            "login_id": login_id,
            "verification_url": verification_url,
            "user_code": user_code,
        }))
        .into_response(),
        Ok(CodexLoginStart::Browser { .. }) => codex_error_response(
            CodexSubscriptionError::Protocol("unexpected browser login response".into()),
        ),
        Err(response) => response,
    }
}

async fn codex_rate_limits(State(state): State<AppState>) -> Response {
    match run_codex(state.codex, |service| service.rate_limits()).await {
        Ok(rate_limits) => Json(rate_limits).into_response(),
        Err(response) => response,
    }
}

async fn codex_logout(State(state): State<AppState>) -> Response {
    match run_codex(state.codex, |service| service.logout()).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(response) => response,
    }
}

async fn run_codex<R, F>(service: Arc<dyn CodexAccountService>, operation: F) -> Result<R, Response>
where
    R: Send + 'static,
    F: FnOnce(&dyn CodexAccountService) -> Result<R, CodexSubscriptionError> + Send + 'static,
{
    match tokio::task::spawn_blocking(move || operation(service.as_ref())).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(codex_error_response(error)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "codex_provider_task_failed"})),
        )
            .into_response()),
    }
}

fn codex_error_response(error: CodexSubscriptionError) -> Response {
    let (status, code) = match error {
        CodexSubscriptionError::ProviderUnavailable(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "codex_provider_unavailable",
        ),
        CodexSubscriptionError::NotInitialized => (
            StatusCode::SERVICE_UNAVAILABLE,
            "codex_provider_not_initialized",
        ),
        CodexSubscriptionError::Protocol(_) => {
            (StatusCode::BAD_GATEWAY, "codex_provider_protocol_error")
        }
    };
    (status, Json(json!({"error": code}))).into_response()
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
        Ok(view) => (StatusCode::ACCEPTED, Json(view)).into_response(),
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
        assert_eq!(record.view.status, ObjectiveStatus::Queued);
    }

    #[tokio::test]
    async fn queued_objective_emits_flat_typed_lifecycle_event() {
        let state = AppState::new(None);
        let mut events = state.events.subscribe();
        let view = state
            .enqueue(ObjectiveRequest {
                repository: "owner/repo".to_string(),
                description: "Emit typed lifecycle event".to_string(),
                branch: Some("autodev/events".to_string()),
            })
            .await
            .expect("enqueue objective");

        let payload = events.recv().await.expect("queued lifecycle event");
        let value: Value = serde_json::from_str(&payload).expect("valid event JSON");
        assert_eq!(
            value,
            json!({
                "type": "objective_queued",
                "objective_id": view.id,
                "task_id": view.current_task_id,
                "phase": Value::Null,
                "status": "queued",
                "evidence_ref": Value::Null,
                "message": "objective accepted"
            })
        );
        assert!(value.get("approval_ref").is_none());
        assert!(value.get("data").is_none());
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
