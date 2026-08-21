use forge_core::{
    ActionType, AgentAction, Capability, ExecutableAction, ExecutionError, RiskLevel, Workspace,
};
use serde_json::json;

/// Build the canonical authorized Cargo `RunTest` action used by integration coverage.
fn run_test_action() -> AgentAction {
    AgentAction {
        id: "run-test-1".to_string(),
        task_id: "task-1".to_string(),
        agent_id: "tester-1".to_string(),
        action_type: ActionType::RunTest,
        reason: "verify repository tests".to_string(),
        risk: RiskLevel::Low,
        capabilities: vec![Capability::RunTest],
        payload: json!({
            "runner": "cargo",
            "args": ["test"]
        }),
        expected: json!({}),
    }
}

#[test]
fn run_test_reaches_the_fail_closed_process_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = Workspace::new(dir.path(), 4096).unwrap();

    let error =
        forge_core::execute(&ExecutableAction::new(run_test_action(), workspace)).unwrap_err();

    assert!(matches!(error, ExecutionError::ProcessSandboxRequired));
}
