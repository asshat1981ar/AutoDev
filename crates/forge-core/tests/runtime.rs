//! Integration tests for the agent runtime: a full step with a mock model and
//! a real (workspace-bound) executor, plus evidence recording.

use forge_core::{
    ActionType, AgentAction, AgentProfile, AgentRole, AgentRuntime, AgentRuntimeState, AgentState,
    Capability, ExecutableAction, ExecutionResult, MockProvider, RiskLevel, Task, Workspace,
};
use serde_json::json;

/// Build a developer profile directly (avoiding the full default set).
fn developer_profile() -> AgentProfile {
    AgentProfile {
        role: AgentRole::Developer,
        description: "developer".to_string(),
        capabilities: vec![Capability::ReadFile, Capability::WriteFile],
        model: forge_core::ModelRequirement {
            family: "ollama".to_string(),
            preferred: "qwen2.5-coder".to_string(),
            min_context_tokens: 8192,
        },
        policy: forge_core::AgentPolicy {
            risk_ceiling: RiskLevel::Medium,
            tools: vec!["read_file".to_string()],
            timeout_secs: 60,
            retry: forge_core::RetryPolicy {
                max_attempts: 2,
                backoff_ms: 100,
            },
        },
        expected_output: "code_change".to_string(),
    }
}

/// An executor bound to a real workspace, so `read_file` really runs.
fn workspace_executor(ws: Workspace) -> forge_core::Executor {
    Box::new(
        move |action: &AgentAction| -> Result<ExecutionResult, forge_core::ExecutionError> {
            forge_core::execute(&ExecutableAction::new(action.clone(), ws.clone()))
        },
    )
}

#[test]
fn full_step_reads_a_file_and_records_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    std::fs::write(dir.path().join("a.txt"), b"hello runtime").unwrap();

    // Mock model returns a read_file action.
    let provider = MockProvider::new(
        json!({
            "action": "read_file",
            "reason": "inspect",
            "risk": "low",
            "payload": { "path": "a.txt" }
        })
        .to_string(),
    );

    let mut rt = AgentRuntime::new(
        "dev-rt",
        developer_profile(),
        Box::new(provider),
        workspace_executor(ws),
    );
    rt.assign_task(Task {
        id: "t1".to_string(),
        title: "read".to_string(),
        context: "read a.txt".to_string(),
    });

    let outcome = rt.run_step().unwrap();
    assert_eq!(outcome.state, AgentRuntimeState::Completed);
    let result = outcome.result.as_ref().unwrap();
    assert_eq!(result.stdout, "hello runtime");
    // Evidence recorded with a verifiable fingerprint.
    assert_eq!(rt.evidence.len(), 1);
    assert!(rt.evidence.records()[0].verify());
    // The produced action is reconstructable from evidence.
    let record = rt.evidence.records()[0].clone();
    let reconstructed: AgentAction = serde_json::from_value(record.record.action.clone()).unwrap();
    assert_eq!(reconstructed.action_type, ActionType::ReadFile);
    assert_eq!(reconstructed.task_id, "t1");
}

#[test]
fn registry_instance_matches_runtime_role() {
    // The registry's AgentInstance carries a role; the runtime mirrors it.
    let profile = developer_profile();
    let rt = AgentRuntime::new(
        "dev-rt",
        profile.clone(),
        Box::new(MockProvider::default()),
        Box::new(|_| {
            Ok(ExecutionResult {
                action_id: "x".to_string(),
                status: forge_core::ExecutionStatus::Succeeded,
                started_at: chrono::Utc::now(),
                completed_at: chrono::Utc::now(),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                artifacts: vec![],
                verification: None,
                error: None,
            })
        }),
    );
    assert_eq!(rt.role, AgentRole::Developer);
    assert_eq!(rt.state, AgentRuntimeState::Ready);
    // AgentState is a separate lifecycle descriptor (registry).
    let _state = AgentState::Instantiated;
}
