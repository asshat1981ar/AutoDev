use autodev_server::{router, AppState, ObjectiveStatus};
use axum::{
    body::{to_bytes, Body},
    http::{header::CONTENT_TYPE, Request, StatusCode},
    response::Response,
};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn body_json(response: Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("JSON response")
}

async fn call(app: axum::Router, request: Request<Body>) -> Response {
    app.oneshot(request).await.expect("response")
}

#[test]
fn objective_status_serializes_as_closed_snake_case_values() {
    let cases = [
        (ObjectiveStatus::Queued, "queued"),
        (ObjectiveStatus::Planning, "planning"),
        (ObjectiveStatus::Running, "running"),
        (ObjectiveStatus::Blocked, "blocked"),
        (ObjectiveStatus::Verifying, "verifying"),
        (ObjectiveStatus::Replanned, "replanned"),
        (ObjectiveStatus::Completed, "completed"),
        (ObjectiveStatus::Failed, "failed"),
    ];

    for (status, expected) in cases {
        assert_eq!(serde_json::to_value(status).unwrap(), json!(expected));
    }
}

#[tokio::test]
async fn created_objective_can_be_fetched_by_id_without_internal_graph_state() {
    let app = router(AppState::new(None));
    let create =
        Request::builder()
            .method("POST")
            .uri("/api/v1/objectives")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "repository": "asshat1981ar/AutoDev",
                    "description": "Implement the Working APK bridge",
                    "branch": "autodev/test-objective"
                })
                .to_string(),
            ))
            .expect("create request");

    let created = call(app.clone(), create).await;
    assert_eq!(created.status(), StatusCode::ACCEPTED);
    let created_json = body_json(created).await;
    let id = created_json["id"].as_str().expect("objective id");
    assert_eq!(created_json["status"], "queued");
    assert!(created_json.get("graph").is_none());
    assert!(created_json.get("current_task_id").is_some());
    assert!(created_json.get("current_phase").is_some());
    assert!(created_json.get("latest_evidence_ref").is_some());
    assert!(created_json.get("blocked_reason").is_some());

    let get =
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/objectives/{id}"))
            .body(Body::empty())
            .expect("get request");
    let fetched = call(app.clone(), get).await;
    assert_eq!(fetched.status(), StatusCode::OK);
    assert_eq!(body_json(fetched).await, created_json);

    let missing =
        Request::builder()
            .method("GET")
            .uri("/api/v1/objectives/missing")
            .body(Body::empty())
            .expect("missing request");
    let missing = call(app, missing).await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_json(missing).await, json!({"error": "objective_not_found"}));
}
