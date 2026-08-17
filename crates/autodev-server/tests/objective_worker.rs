use std::sync::Arc;

use autodev_server::{
    router, run_objective_cycle, ActionProposer, AppState, ModelActionProposer, ObjectiveRunner,
    ObjectiveSnapshot, ObjectiveStatus, ObjectiveStore, ObjectiveView, RunnerError,
    RunnerExecution,
};
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use forge_core::{
    default_profiles, mock_verifier, ActionProposal, ActionType, AgentAction, AgentRole,
    MockProvider, PolicyDecision, RiskLevel, TaskGraph, VerificationFabric, VerificationKind,
    VerifiedOrchestratorState, Workspace,
};
use serde_json::json;
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

struct SelectiveProposer;

impl ActionProposer for SelectiveProposer {
    fn propose(&self, task: &forge_core::TaskNode) -> Result<ActionProposal, RunnerError> {
        if task.description.contains("fail proposal") {
            return Err(RunnerError::NoReadyTask);
        }
        Ok(ActionProposal {
            action: AgentAction {
                id: format!("action-{}", task.id),
                task_id: task.id.clone(),
                agent_id: "developer".to_string(),
                action_type: ActionType::ReadFile,
                reason: "inspect repository".to_string(),
                risk: RiskLevel::Low,
                capabilities: vec![],
                payload: json!({"path": "README.md"}),
                expected: json!({}),
            },
            decision: PolicyDecision::Allow,
            model: "test-model".to_string(),
        })
    }
}

fn queued_snapshot(id: &str, description: &str) -> ObjectiveSnapshot {
    let graph = TaskGraph::single(id, description);
    ObjectiveSnapshot {
        view: ObjectiveView {
            id: id.to_string(),
            repository: "asshat1981ar/AutoDev".to_string(),
            description: description.to_string(),
            branch: format!("autodev/{id}"),
            status: ObjectiveStatus::Queued,
            current_task_id: Some(graph.root.clone()),
            current_phase: None,
            latest_evidence_ref: None,
            blocked_reason: None,
        },
        graph,
        orchestrator: VerifiedOrchestratorState::default(),
    }
}

#[test]
fn worker_cycle_advances_later_objectives_even_when_an_earlier_objective_errors() {
    let state = AppState::new(None);
    let store = state.store();
    store
        .put(&queued_snapshot("a-failing", "fail proposal"))
        .unwrap();
    store
        .put(&queued_snapshot("b-working", "valid proposal"))
        .unwrap();
    let runner = ObjectiveRunner::new(
        store.clone(),
        Arc::new(SelectiveProposer),
        state.event_sender(),
    );

    assert_eq!(run_objective_cycle(&runner).unwrap(), 2);
    assert_eq!(run_objective_cycle(&runner).unwrap(), 2);
    assert!(run_objective_cycle(&runner).is_err());

    let later = store.get("b-working").unwrap().unwrap();
    assert!(
        later.graph.root().planned_action.is_some(),
        "a failing objective must not starve later objectives in the same worker cycle"
    );
}
