use std::{collections::BTreeMap, convert::Infallible, sync::Arc};

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Request, State},
    http::{header, HeaderMap, StatusCode, Uri},
    middleware::{self, Next},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use forge_core::{ExecPlan, PlanBudget, PlanMilestone, TaskGraph};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use uuid::Uuid;

mod mcp;

type HmacSha256 = Hmac<Sha256>;
const MCP_BEARER_COMPARE_KEY: &[u8] = b"autodev-mcp-bearer-constant-time-compare-v1";
const DEFAULT_MCP_ORIGIN_HOSTS: [&str; 4] = ["localhost", "127.0.0.1", "::1", "autodev-server"];

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
    exec_plans: Arc<RwLock<BTreeMap<String, ExecPlan>>>,
    events: broadcast::Sender<String>,
    github_webhook_secret: Option<String>,
    mcp_bearer_tag: Option<Arc<Vec<u8>>>,
}

impl AppState {
    pub fn new(github_webhook_secret: Option<String>) -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            objectives: Arc::new(RwLock::new(BTreeMap::new())),
            exec_plans: Arc::new(RwLock::new(BTreeMap::new())),
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

    /// Return a cloned durable coordination plan without granting execution authority.
    pub async fn exec_plan(&self, plan_id: &str) -> Option<ExecPlan> {
        self.exec_plans.read().await.get(plan_id).cloned()
    }

    async fn enqueue(&self, request: ObjectiveRequest) -> Result<ObjectiveRecord, &'static str> {
        if request.repository.trim().is_empty() {
            return Err("repository is required");
        }
        if request.description.trim().is_empty() {
            return Err("description is required");
        }

        let id = Uuid::new_v4().to_string();
        let description = request.description.trim().to_string();
        let branch = request
            .branch
            .filter(|branch| !branch.trim().is_empty())
            .unwrap_or_else(|| format!("autodev/objective-{}", &id[..8]));
        let graph = TaskGraph::single(&format!("Objective {id}"), &description);
        let record = ObjectiveRecord {
            id: id.clone(),
            repository: request.repository.trim().to_string(),
            description: description.clone(),
            branch,
            status: "queued".to_string(),
            graph,
        };

        let mut plan = ExecPlan::new(id.clone(), description, PlanBudget::new(3, 3));
        plan.add_milestone(PlanMilestone::new("objective", "Complete objective"))
            .map_err(|_| "failed to initialize exec plan")?;

        self.objectives
            .write()
            .await
            .insert(id.clone(), record.clone());
        self.exec_plans.write().await.insert(id.clone(), plan);
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
        .layer(middleware::from_fn_with_state(state, require_mcp_bearer))
        .layer(middleware::from_fn(require_mcp_origin));

    api.merge(mcp)
}

async fn require_mcp_origin(request: Request, next: Next) -> Response {
    let Some(origin) = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    else {
        return next.run(request).await;
    };

    if !mcp_origin_allowed(origin) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "untrusted MCP Origin"})),
        )
            .into_response();
    }

    next.run(request).await
}

fn mcp_origin_allowed(origin: &str) -> bool {
    let Ok(uri) = origin.parse::<Uri>() else {
        return false;
    };
    if !matches!(uri.scheme_str(), Some("http" | "https")) {
        return false;
    }
    let Some(authority) = uri.authority() else {
        return false;
    };

    configured_mcp_origin_hosts()
        .iter()
        .any(|allowed| authority.host().eq_ignore_ascii_case(allowed))
}

fn configured_mcp_origin_hosts() -> Vec<String> {
    std::env::var("AUTODEV_MCP_ALLOWED_HOSTS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|host| !host.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|hosts| !hosts.is_empty())
        .unwrap_or_else(|| {
            DEFAULT_MCP_ORIGIN_HOSTS
                .iter()
                .map(|host| host.to_string())
                .collect()
        })
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
        .expect("static MCP bearer comparison key should be valid");
    mac.update(token.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn verify_mcp_bearer(expected_tag: &[u8], presented: &str) -> bool {
    let mut mac = HmacSha256::new_from_slice(MCP_BEARER_COMPARE_KEY)
        .expect("static MCP bearer comparison key should be valid");
    mac.update(presented.as_bytes());
    mac.verify_slice(expected_tag).is_ok()
}

async fn health() -> &'static str {
    "ok"
}

async fn create_objective(
    State(state): State<AppState>,
    Json(request): Json<ObjectiveRequest>,
) -> Result<(StatusCode, Json<ObjectiveRecord>), (StatusCode, Json<Value>)> {
    let record = state.enqueue(request).await.map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": message})),
        )
    })?;
    Ok((StatusCode::ACCEPTED, Json(record)))
}

async fn list_objectives(State(state): State<AppState>) -> Json<Vec<ObjectiveRecord>> {
    Json(state.objectives.read().await.values().cloned().collect())
}

async fn event_stream(State(state): State<AppState>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.events.subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(|message| match message {
        Ok(message) => Some(Ok(Event::default().data(message))),
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn github_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let Some(secret) = state.github_webhook_secret.as_deref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "GITHUB_WEBHOOK_SECRET is not configured"})),
        )
            .into_response();
    };

    if !verify_github_signature(secret, &headers, &body) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid signature"})),
        )
            .into_response();
    }

    (
        StatusCode::ACCEPTED,
        Json(json!({"accepted": true, "bytes": body.len()})),
    )
        .into_response()
}

fn verify_github_signature(secret: &str, headers: &HeaderMap, body: &[u8]) -> bool {
    let Some(signature) = headers
        .get("x-hub-signature-256")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("sha256="))
    else {
        return false;
    };
    let Ok(provided) = hex::decode(signature) else {
        return false;
    };

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(body);
    mac.verify_slice(&provided).is_ok()
}
