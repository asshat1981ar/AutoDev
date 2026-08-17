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
        .oneshot(modern_request("discover-no-secret", "server/discover", json!({})))
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
