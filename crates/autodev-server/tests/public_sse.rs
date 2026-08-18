use autodev_server::{router, AppState};
use axum::{
    body::{Body, Bytes},
    http::Request,
};
use serde_json::Value;
use tokio_stream::StreamExt;
use tower::ServiceExt;

fn decode_sse_data(frame: Bytes) -> Value {
    let text = String::from_utf8(frame.to_vec()).expect("utf-8 SSE frame");
    let data = text
        .lines()
        .find_map(|line| line.strip_prefix("data: "))
        .expect("SSE data line");
    serde_json::from_str(data).expect("JSON SSE data")
}

#[tokio::test]
async fn objective_queue_event_uses_public_v1_envelope() {
    let app = router(AppState::new(Some("secret".to_string())));

    let stream_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/events")
                .body(Body::empty())
                .expect("stream request"),
        )
        .await
        .expect("stream response");
    assert_eq!(stream_response.status(), 200);
    let mut body_stream = stream_response.into_body().into_data_stream();

    let create_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/objectives")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"repository":"owner/repo","description":"Implement health endpoint"}"#,
                ))
                .expect("create request"),
        )
        .await
        .expect("create response");
    assert_eq!(create_response.status(), 202);

    let frame = body_stream
        .next()
        .await
        .expect("first SSE frame")
        .expect("SSE body chunk");
    let event = decode_sse_data(frame);

    assert_eq!(event["schema_version"], "1");
    assert_eq!(event["type"], "objective_queued");
    assert!(event["event_id"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert!(event["timestamp"]
        .as_str()
        .is_some_and(|value| value.ends_with('Z')));
    assert!(event["objective_id"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert_eq!(event["run_id"], Value::Null);
    assert_eq!(event["task_id"], Value::Null);
    assert_eq!(event["data"]["repository"], "owner/repo");
    assert_eq!(event["data"]["status"], "queued");
}
