use autodev_server::{router, AppState};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn control_plane_accepts_objectives_and_rejects_unsigned_webhooks() {
    let app = router(AppState::new(Some("secret".to_string())));
    let objective = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/objectives")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"repository":"owner/repo","description":"Implement health endpoint"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(objective.status(), StatusCode::ACCEPTED);

    let unsigned = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header("x-github-event", "issues")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unsigned.status(), StatusCode::UNAUTHORIZED);
}
