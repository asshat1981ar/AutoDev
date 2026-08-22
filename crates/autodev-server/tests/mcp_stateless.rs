use autodev_server::{router, AppState};
use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

const TEST_TOKEN: &str = "test-mcp-token";

fn secured_state() -> AppState {
    AppState::new(None).with_mcp_bearer_token(TEST_TOKEN)
}

fn modern_request(id: &str, method: &str, params: Value) -> Request<Body> {
    modern_request_with_token(id, method, params, Some(TEST_TOKEN))
}

fn modern_request_with_token(
    id: &str,
    method: &str,
    params: Value,
    token: Option<&str>,
) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header(header::HOST, "localhost")
        .header(header::ACCEPT, "application/json, text/event-stream")
        .header(header::CONTENT_TYPE, "application/json")
        .header("MCP-Protocol-Version", "2026-07-28")
        .header("Mcp-Method", method);
    if method == "tools/call" {
        if let Some(name) = params.get("name").and_then(Value::as_str) {
            builder = builder.header("Mcp-Name", name);
        }
    }
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder
        .body(Body::from(
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
                "params": {
                    "_meta": {
                        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                        "io.modelcontextprotocol/clientInfo": {
                            "name": "autodev-test",
                            "version": "0.1.0"
                        },
                        "io.modelcontextprotocol/clientCapabilities": {}
                    }
                }
            })
            .as_object()
            .map(|base| {
                let mut request = Value::Object(base.clone());
                if let Some(request_params) =
                    request.get_mut("params").and_then(Value::as_object_mut)
                {
                    if let Some(extra) = params.as_object() {
                        for (key, value) in extra {
                            request_params.insert(key.clone(), value.clone());
                        }
                    }
                }
                request
            })
            .expect("request envelope is an object")
            .to_string(),
        ))
        .expect("valid request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("bounded response body");
    serde_json::from_slice(&bytes).expect("MCP response should be JSON")
}

#[tokio::test]
async fn mcp_fails_closed_when_bearer_secret_is_not_configured() {
    let response = router(AppState::new(None))
        .oneshot(modern_request(
            "discover-no-secret",
            "server/discover",
            json!({}),
        ))
        .await
        .expect("router response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn mcp_rejects_an_invalid_bearer_token() {
    let response = router(secured_state())
        .oneshot(modern_request_with_token(
            "discover-bad-token",
            "server/discover",
            json!({}),
            Some("wrong-token"),
        ))
        .await
        .expect("router response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn server_discover_uses_modern_stateless_transport() {
    let response = router(secured_state())
        .oneshot(modern_request("discover-1", "server/discover", json!({})))
        .await
        .expect("router response");

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("Mcp-Session-Id").is_none());
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );

    let body = json_body(response).await;
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], "discover-1");
    assert!(body["result"]["supportedVersions"]
        .as_array()
        .is_some_and(|versions| versions.iter().any(|version| version == "2026-07-28")));
    assert!(body["result"]["capabilities"]["tools"].is_object());
}

#[tokio::test]
async fn tools_list_works_without_a_prior_discover_request() {
    let response = router(secured_state())
        .oneshot(modern_request("tools-1", "tools/list", json!({})))
        .await
        .expect("router response");

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("Mcp-Session-Id").is_none());

    let body = json_body(response).await;
    let names: Vec<&str> = body["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();

    assert!(names.contains(&"autodev.objectives.list"));
    assert!(names.contains(&"autodev.gaps.scan"));
    assert!(names.contains(&"autodev.action.propose"));
}

#[tokio::test]
async fn action_proposal_is_untrusted_and_never_authorizes_execution() {
    let response = router(secured_state())
        .oneshot(modern_request(
            "proposal-1",
            "tools/call",
            json!({
                "name": "autodev.action.propose",
                "arguments": {
                    "task_id": "task-1",
                    "agent_id": "mcp-client",
                    "reason": "stage evaluated candidate",
                    "path": ".cline/candidates/skills/example/SKILL.md",
                    "content": "candidate"
                }
            }),
        ))
        .await
        .expect("router response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let text = body["result"]["content"][0]["text"]
        .as_str()
        .expect("proposal tool should return text content");
    let proposal: Value = serde_json::from_str(text).expect("proposal should be AgentAction JSON");

    assert_eq!(proposal["type"], "write_file");
    assert_eq!(proposal["payload"]["operation"], "write_file");
    assert_eq!(
        proposal["payload"]["path"],
        ".cline/candidates/skills/example/SKILL.md"
    );
    assert_eq!(proposal["expected"]["status"], "candidate_only");
    assert_eq!(proposal["expected"]["execution_authorized"], false);
    assert_eq!(proposal["payload"]["approved"], Value::Null);
}

#[tokio::test]
async fn action_proposal_rejects_path_traversal_before_forgecore() {
    let response = router(secured_state())
        .oneshot(modern_request(
            "proposal-traversal",
            "tools/call",
            json!({
                "name": "autodev.action.propose",
                "arguments": {
                    "task_id": "task-1",
                    "agent_id": "mcp-client",
                    "reason": "invalid candidate",
                    "path": "../escape",
                    "content": "candidate"
                }
            }),
        ))
        .await
        .expect("router response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"]["code"], -32602);
    assert!(body["result"].is_null());
}

#[tokio::test]
async fn repeated_requests_do_not_create_protocol_sessions() {
    let app = router(secured_state());

    for id in ["discover-a", "discover-b"] {
        let response = app
            .clone()
            .oneshot(modern_request(id, "server/discover", json!({})))
            .await
            .expect("router response");

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get("Mcp-Session-Id").is_none());
        let body = json_body(response).await;
        assert_eq!(body["id"], id);
    }
}

// ----- Adversarial coverage additions for the hardening batch -----
//
// These tests target the contract surfaces called out in the
// `autodev-server` security audit: HMAC prefix/length failure modes,
// oversized-body rejection on the public API surface, and the
// bearer-not-configured empty-secret path returning 503.

/// `AppState::new(Some(""))` must treat the empty secret as "not
/// configured" and the webhook route must return 503. Otherwise an
/// operator who forgets the env var would get a service that silently
/// rejects every webhook with 401 instead of telling them the secret
/// is missing.
#[tokio::test]
async fn webhook_returns_503_when_secret_is_empty() {
    use autodev_server::AppState;
    let app = router(AppState::new(Some(String::new())));
    let body = serde_json::json!({
        "action": "opened",
        "issue": {"number": 1, "title": "x"}
    })
    .to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .expect("router response");
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

/// HMAC verification must reject a header that does not start with
/// `sha256=`. A header of bare hex without the prefix is a common
/// misconfiguration and must not pass.
#[tokio::test]
async fn webhook_rejects_signature_without_sha256_prefix() {
    use autodev_server::AppState;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let secret = "test-webhook-secret";
    let app = router(AppState::new(Some(secret.to_string())));
    let body = b"{\"action\":\"opened\"}";
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    let bare_hex = format!("{:x}", mac.finalize().into_bytes()); // no `sha256=` prefix

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header("X-Hub-Signature-256", bare_hex)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_vec()))
                .unwrap(),
        )
        .await
        .expect("router response");
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "bare hex (no `sha256=` prefix) must be rejected as 401, not 202"
    );
}

/// HMAC verification must reject a hex string of the wrong length.
/// A truncated or padded signature must not be accepted.
#[tokio::test]
async fn webhook_rejects_signature_of_wrong_length() {
    use autodev_server::AppState;
    let app = router(AppState::new(Some("test-webhook-secret".to_string())));
    let body = b"{\"action\":\"opened\"}";

    // 31 bytes of hex instead of 32.
    let short_hex = "a".repeat(62);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header("X-Hub-Signature-256", format!("sha256={short_hex}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_vec()))
                .unwrap(),
        )
        .await
        .expect("router response");
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "wrong-length hex signature must be rejected"
    );
}

/// The public API surface (`/api/v1/objectives`) must reject a body
/// larger than `API_MAX_BODY_BYTES`. This bounds memory pressure from
/// unauthenticated callers, including the LAN-bearer exposure case
/// noted in the deployment contract.
#[tokio::test]
async fn api_objectives_rejects_oversized_body() {
    use autodev_server::AppState;
    let app = router(AppState::new(None));
    // `API_MAX_BODY_BYTES` is 512 KiB; send one byte more.
    let oversized = "x".repeat(autodev_server::API_MAX_BODY_BYTES + 1);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/objectives")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(oversized))
                .unwrap(),
        )
        .await
        .expect("router response");
    assert_eq!(
        response.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "oversized body on /api/v1/objectives must be 413"
    );
}

/// `with_mcp_bearer_token("")` is equivalent to no token configured;
/// the `/mcp` route must return 503, not 401. An empty token would
/// otherwise be accepted by a `Bearer ` (empty) presentation on a
/// per-call check.
#[tokio::test]
async fn mcp_returns_503_when_bearer_token_is_empty() {
    use autodev_server::AppState;
    let app = router(AppState::new(None).with_mcp_bearer_token(String::new()));
    let response = app
        .oneshot(modern_request(
            "discover-empty-token",
            "server/discover",
            json!({}),
        ))
        .await
        .expect("router response");
    assert_eq!(
        response.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "empty bearer token must be treated as 'not configured'"
    );
}
