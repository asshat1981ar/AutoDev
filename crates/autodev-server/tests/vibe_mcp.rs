use autodev_server::{router, AppState};
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn objective_intake_creates_observable_durable_exec_plan() {
    let state = AppState::new(Some("secret".to_string()));
    let app = router(state.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/objectives")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"repository":"owner/repo","description":"Expose Vibe MCP development state"}"#,
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), 202);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("response body");
    let objective: Value = serde_json::from_slice(&body).expect("objective JSON");
    let objective_id = objective["id"].as_str().expect("objective id");

    let plan = state
        .exec_plan(objective_id)
        .await
        .expect("durable ExecPlan projection");

    assert_eq!(plan.id(), objective_id);
    assert_eq!(plan.goal(), "Expose Vibe MCP development state");
    assert_eq!(plan.budget().max_replans(), 3);
    assert_eq!(plan.budget().max_attempts_per_milestone(), 3);
    assert_eq!(plan.budget().replans_used(), 0);
    assert_eq!(plan.milestones().len(), 1);
    assert_eq!(plan.milestones()[0].id(), "objective");
    assert!(!plan.milestones()[0].is_completed());
}

#[tokio::test]
async fn mcp_rejects_untrusted_browser_origin_before_dispatch() {
    let state = AppState::new(None).with_mcp_bearer_token("test-token");
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("host", "localhost")
                .header("origin", "https://evil.example")
                .header("authorization", "Bearer test-token")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), 403);
}

#[tokio::test]
async fn mcp_rejects_untrusted_origin_before_missing_bearer() {
    let state = AppState::new(None).with_mcp_bearer_token("test-token");
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("host", "localhost")
                .header("origin", "https://evil.example")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), 403);
}
