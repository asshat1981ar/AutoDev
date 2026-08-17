use autodev_server::{router, AppState};
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use serde_json::Value;
use tower::ServiceExt;

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("JSON response")
}

#[tokio::test]
async fn create_objective_returns_public_summary_without_task_graph() {
    let app = router(AppState::new(Some("secret".to_string())));
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

    assert_eq!(response.status(), 202);
    let value = json_body(response).await;
    assert_eq!(value["schema_version"], "1");
    assert_eq!(value["repository"], "owner/repo");
    assert_eq!(value["description"], "Implement health endpoint");
    assert_eq!(value["status"], "queued");
    assert!(value["id"].as_str().is_some_and(|id| !id.is_empty()));
    assert!(value["branch"].as_str().is_some_and(|branch| !branch.is_empty()));
    assert!(value.get("graph").is_none());
    assert!(value.get("capabilities").is_none());
    assert!(value.get("approval_ref").is_none());
}

#[tokio::test]
async fn list_objectives_returns_public_summaries_without_task_graph() {
    let app = router(AppState::new(Some("secret".to_string())));
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/objectives")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"repository":"owner/repo","description":"Inspect state"}"#,
                ))
                .expect("create request"),
        )
        .await
        .expect("create response");
    assert_eq!(create_response.status(), 202);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/objectives")
                .body(Body::empty())
                .expect("list request"),
        )
        .await
        .expect("list response");
    assert_eq!(response.status(), 200);

    let value = json_body(response).await;
    let items = value.as_array().expect("objective array");
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item["schema_version"], "1");
    assert_eq!(item["repository"], "owner/repo");
    assert_eq!(item["description"], "Inspect state");
    assert_eq!(item["status"], "queued");
    assert!(item.get("graph").is_none());
    assert!(item.get("capabilities").is_none());
    assert!(item.get("approval_ref").is_none());
}
