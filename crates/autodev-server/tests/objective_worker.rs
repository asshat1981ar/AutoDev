use std::sync::Arc;

use autodev_server::{
    router, run_objective_cycle, AppState, ModelActionProposer, ObjectiveRunner, ObjectiveStatus,
    ObjectiveStore, ObjectiveView, RunnerExecution,
};
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use forge_core::{
    default_profiles, mock_verifier, AgentRole, MockProvider, VerificationFabric, VerificationKind,
    Workspace,
};
use tempfile::tempdir;
use tower::ServiceExt;

#[tokio::test]
async fn worker_cycles_newly_posted_objective_without_second_http_request() {
    let workspace_dir = tempdir().unwrap();
    std::fs::write(workspace_dir.path().join("README.md"), "hello").unwrap();
    let workspace = Workspace::new(workspace_dir.path(), 1024 * 1024).unwrap();

    let state = AppState::new(None);
    let store = state.store();
    let developer = default_profiles()
        .into_iter()
        .find(|profile| profile.role == AgentRole::Developer)
        .expect("developer profile");
    let provider = Arc::new(MockProvider::new(
        serde_json::json!({
            "action": "read_file",
            "reason": "inspect repository",
            "risk": "low",
            "payload": {"path": "README.md"}
        })
        .to_string(),
    ));
    let proposer = Arc::new(ModelActionProposer::new("developer", developer, provider));
    let verification = Arc::new(|| {
        VerificationFabric::new().with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, true),
        )
    });
    let runner =
        ObjectiveRunner::new(store.clone(), proposer, state.event_sender()).with_execution(
            RunnerExecution::new(workspace, AgentRole::Developer, verification),
        );

    let response = router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/objectives")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "repository": "asshat1981ar/AutoDev",
                        "description": "Inspect repository before changing it"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: ObjectiveView = serde_json::from_slice(&body).unwrap();
    assert_eq!(created.status, ObjectiveStatus::Queued);

    for _ in 0..4 {
        assert_eq!(run_objective_cycle(&runner).unwrap(), 1);
    }

    let completed = store.get(&created.id).unwrap().unwrap();
    assert_eq!(completed.view.status, ObjectiveStatus::Completed);
    assert!(completed.view.latest_evidence_ref.is_some());
    assert_eq!(completed.orchestrator.envelopes.len(), 1);
}
