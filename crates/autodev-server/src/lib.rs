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
use forge_core::{
    attest_gate_now, verify_attestation, GateAttestation, GateAttestationRequest, GateSigningKey,
    TaskGraph,
};
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
const API_BEARER_COMPARE_KEY: &[u8] = b"autodev-api-bearer-constant-time-compare-v1";

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
    mcp_bearer_tag: Option<Arc<Vec<u8>>>,
    api_bearer_tag: Option<Arc<Vec<u8>>>,
    gate_key: Option<Arc<GateSigningKey>>,
    attestations: Arc<RwLock<BTreeMap<String, GateAttestation>>>,
}

/// Upper bound on attestations kept in memory. Persistence belongs to the
/// evidence store (ADR-002); this cache only answers `recorded` on verify.
pub const MAX_RECORDED_ATTESTATIONS: usize = 10_000;

impl AppState {
    pub fn new(github_webhook_secret: Option<String>) -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            objectives: Arc::new(RwLock::new(BTreeMap::new())),
            events,
            github_webhook_secret: github_webhook_secret.filter(|value| !value.trim().is_empty()),
            mcp_bearer_tag: None,
            api_bearer_tag: None,
            gate_key: None,
            attestations: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Hold the gate-attestation signing key (ADR-007). Without it, the gate
    /// routes fail closed with `503 Service Unavailable`.
    pub fn with_gate_signing_key(mut self, key: GateSigningKey) -> Self {
        self.gate_key = Some(Arc::new(key));
        self
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

    /// Configure an optional bearer token for mutating public API routes.
    ///
    /// When unset, objective creation remains available for backward-compatible
    /// loopback deployments. Operators that expose the API beyond loopback must
    /// configure this token at the reverse proxy and application boundary.
    pub fn with_api_bearer_token(mut self, token: impl Into<String>) -> Self {
        let token = token.into();
        self.api_bearer_tag = if token.trim().is_empty() {
            None
        } else {
            Some(Arc::new(bearer_tag(API_BEARER_COMPARE_KEY, &token)))
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
    let mutating_api = Router::new()
        .route("/api/v1/objectives", post(create_objective))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_bearer_if_configured,
        ));

    // Gate attestation is never anonymous: unlike objective intake, these
    // routes require the API bearer even on loopback (ADR-007).
    let gate_api = Router::new()
        .route("/api/v1/gate/attest", post(attest_gate_route))
        .route("/api/v1/gate/verify", post(verify_gate_route))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_bearer_strict,
        ));

    let api = Router::new()
        .route("/health", get(health))
        .route("/api/v1/objectives", get(list_objectives))
        .route("/api/v1/events/stream", get(event_stream))
        .route("/events", get(event_stream))
        .route("/webhooks/github", post(github_webhook))
        .merge(mutating_api)
        .merge(gate_api)
        // Bound the request body on the public-API surface. The default
        // Axum limit is 2 MiB, which is more than a webhook or an
        // objective description should ever be; the limit mirrors the
        // /mcp cap to keep memory pressure bounded.
        .layer(DefaultBodyLimit::max(API_MAX_BODY_BYTES))
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

    if !presented.is_some_and(|token| verify_bearer(MCP_BEARER_COMPARE_KEY, expected_tag, token)) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid MCP bearer token"})),
        )
            .into_response();
    }

    next.run(request).await
}

async fn require_api_bearer_if_configured(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let Some(expected_tag) = state.api_bearer_tag.as_deref() else {
        return next.run(request).await;
    };
    let presented = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty());
    if !presented.is_some_and(|token| verify_bearer(API_BEARER_COMPARE_KEY, expected_tag, token)) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid API bearer token"})),
        )
            .into_response();
    }
    next.run(request).await
}

async fn require_api_bearer_strict(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if state.api_bearer_tag.is_none() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "AUTODEV_API_BEARER_TOKEN is required for gate attestation"})),
        )
            .into_response();
    }
    require_api_bearer_if_configured(State(state), request, next).await
}

async fn attest_gate_route(
    State(state): State<AppState>,
    Json(request): Json<GateAttestationRequest>,
) -> Response {
    let Some(key) = state.gate_key.as_deref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "AUTODEV_GATE_SIGNING_KEY is not configured"})),
        )
            .into_response();
    };
    let attestation = match attest_gate_now(&request, key) {
        Ok(attestation) => attestation,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": error.to_string()})),
            )
                .into_response()
        }
    };
    {
        let mut store = state.attestations.write().await;
        while store.len() >= MAX_RECORDED_ATTESTATIONS {
            store.pop_first();
        }
        store.insert(attestation.attestation_id.clone(), attestation.clone());
    }
    let _ = state.events.send(
        json!({
            "type": "gate_attested",
            "data": {
                "attestation_id": attestation.attestation_id,
                "repository": attestation.repository,
                "target_ref": attestation.target_ref,
                "tool": attestation.tool,
                "decision": attestation.decision,
            }
        })
        .to_string(),
    );
    (StatusCode::CREATED, Json(attestation)).into_response()
}

async fn verify_gate_route(
    State(state): State<AppState>,
    Json(attestation): Json<GateAttestation>,
) -> Response {
    let Some(key) = state.gate_key.as_deref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "AUTODEV_GATE_SIGNING_KEY is not configured"})),
        )
            .into_response();
    };
    let valid = verify_attestation(&attestation, key);
    let recorded = valid
        && state
            .attestations
            .read()
            .await
            .get(&attestation.attestation_id)
            .is_some_and(|stored| stored == &attestation);
    Json(json!({
        "valid": valid,
        "recorded": recorded,
        "attestation_id": attestation.attestation_id,
    }))
    .into_response()
}

/// Maximum body size accepted on the public API surface
/// (`/api/v1/*`, `/events`, `/webhooks/*`). Matches the MCP cap so an
/// attacker cannot drive memory pressure by sending oversized payloads to
/// the unauthenticated routes.
pub const API_MAX_BODY_BYTES: usize = 512 * 1024;

pub fn normalize_api_bearer_token(token: Option<String>) -> Option<String> {
    token
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn mcp_bearer_tag(token: &str) -> Vec<u8> {
    bearer_tag(MCP_BEARER_COMPARE_KEY, token)
}

fn bearer_tag(key: &[u8], token: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("constant bearer comparison key is valid");
    mac.update(token.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn verify_bearer(key: &[u8], expected_tag: &[u8], presented: &str) -> bool {
    let Ok(mut mac) = HmacSha256::new_from_slice(key) else {
        return false;
    };
    mac.update(presented.as_bytes());
    mac.verify_slice(expected_tag).is_ok()
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

    #[test]
    fn mcp_bearer_verification_is_constant_time_over_tags() {
        let expected = mcp_bearer_tag("secret-token");
        assert!(verify_bearer(
            MCP_BEARER_COMPARE_KEY,
            &expected,
            "secret-token"
        ));
        assert!(!verify_bearer(
            MCP_BEARER_COMPARE_KEY,
            &expected,
            "wrong-token"
        ));
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
    async fn objective_intake_requires_configured_api_bearer() {
        let state = AppState::new(None).with_api_bearer_token("api-secret");
        let request = || {
            Request::builder()
                .method("POST")
                .uri("/api/v1/objectives")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"repository":"owner/repo","description":"Implement health endpoint"}"#,
                ))
                .expect("request")
        };

        let missing = router(state.clone())
            .oneshot(request())
            .await
            .expect("response");
        assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);

        let wrong = router(state.clone())
            .oneshot({
                let mut request = request();
                request.headers_mut().insert(
                    header::AUTHORIZATION,
                    "Bearer wrong".parse().expect("header"),
                );
                request
            })
            .await
            .expect("response");
        assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);

        let accepted = router(state.clone())
            .oneshot({
                let mut request = request();
                request.headers_mut().insert(
                    header::AUTHORIZATION,
                    "Bearer api-secret".parse().expect("header"),
                );
                request
            })
            .await
            .expect("response");
        assert_eq!(accepted.status(), StatusCode::ACCEPTED);
        assert_eq!(state.objectives.read().await.len(), 1);

        let listing = router(state)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/objectives")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(listing.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn blank_api_bearer_token_leaves_loopback_compatibility_unconfigured() {
        let state = AppState::new(None).with_api_bearer_token("  \t  ");
        let response = router(state)
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
    }

    #[test]
    fn api_bearer_token_normalization_rejects_blank_values() {
        assert_eq!(normalize_api_bearer_token(None), None);
        assert_eq!(normalize_api_bearer_token(Some(" \t ".to_string())), None);
        assert_eq!(
            normalize_api_bearer_token(Some("  api-secret  ".to_string())),
            Some("api-secret".to_string())
        );
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

    fn gate_state() -> AppState {
        AppState::new(None)
            .with_api_bearer_token("api-secret")
            .with_gate_signing_key(GateSigningKey::new([5u8; 32]).expect("key"))
    }

    fn gate_post(uri: &str, bearer: Option<&str>, body: String) -> Request<Body> {
        let mut builder = Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json");
        if let Some(token) = bearer {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }
        builder.body(Body::from(body)).expect("request")
    }

    async fn json_body(response: Response) -> Value {
        let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
            .await
            .expect("body");
        serde_json::from_slice(&bytes).expect("json")
    }

    const PROTECTED_PUSH: &str = r#"{"repository":"owner/repo","tool":"push_files","target_ref":"refs/heads/main","mode":"warn","session_id":"s-1"}"#;

    #[tokio::test]
    async fn gate_attest_fails_closed_without_key_or_api_bearer() {
        let no_key = router(AppState::new(None).with_api_bearer_token("api-secret"));
        let response = no_key
            .oneshot(gate_post(
                "/api/v1/gate/attest",
                Some("api-secret"),
                PROTECTED_PUSH.into(),
            ))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let no_bearer = router(
            AppState::new(None).with_gate_signing_key(GateSigningKey::new([5u8; 32]).expect("key")),
        );
        let response = no_bearer
            .oneshot(gate_post(
                "/api/v1/gate/attest",
                None,
                PROTECTED_PUSH.into(),
            ))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn gate_attest_rejects_wrong_bearer() {
        let response = router(gate_state())
            .oneshot(gate_post(
                "/api/v1/gate/attest",
                Some("wrong"),
                PROTECTED_PUSH.into(),
            ))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn gate_attest_blocks_protected_push_and_verifies_round_trip() {
        let state = gate_state();
        let response = router(state.clone())
            .oneshot(gate_post(
                "/api/v1/gate/attest",
                Some("api-secret"),
                PROTECTED_PUSH.into(),
            ))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::CREATED);
        let attestation = json_body(response).await;
        assert_eq!(attestation["decision"], "BLOCK");
        assert_eq!(attestation["target_ref"], "main");

        let verified = router(state.clone())
            .oneshot(gate_post(
                "/api/v1/gate/verify",
                Some("api-secret"),
                attestation.to_string(),
            ))
            .await
            .expect("response");
        let verdict = json_body(verified).await;
        assert_eq!(verdict["valid"], true);
        assert_eq!(verdict["recorded"], true);

        let mut forged = attestation.clone();
        forged["decision"] = json!("PASS");
        let rejected = router(state)
            .oneshot(gate_post(
                "/api/v1/gate/verify",
                Some("api-secret"),
                forged.to_string(),
            ))
            .await
            .expect("response");
        let verdict = json_body(rejected).await;
        assert_eq!(verdict["valid"], false);
        assert_eq!(verdict["recorded"], false);
    }

    #[tokio::test]
    async fn gate_attest_rejects_client_supplied_decision() {
        let body = r#"{"repository":"owner/repo","tool":"push_files","target_ref":"feat/x","mode":"enforce","session_id":"s","decision":"PASS"}"#;
        let response = router(gate_state())
            .oneshot(gate_post(
                "/api/v1/gate/attest",
                Some("api-secret"),
                body.into(),
            ))
            .await
            .expect("response");
        assert!(response.status().is_client_error());
    }
}
