//! Integration tests for the agent runtime: a full step with a mock model and
//! injected executor, plus evidence recording.
//!
//! The runtime consumes an `Executor`; it does not mint ForgeCore execution
//! authority. Trusted execution behavior is covered by the kernel/orchestrator tests.

use forge_core::{
    ActionType, AgentAction, AgentProfile, AgentRole, AgentRuntime, AgentRuntimeState, AgentState,
    Capability, ExecutionResult, MockProvider, RiskLevel, Task,
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

fn successful_executor() -> forge_core::Executor {
    Box::new(
        |action: &AgentAction| -> Result<ExecutionResult, forge_core::ExecutionError> {
            Ok(ExecutionResult {
                action_id: action.id.clone(),
                status: forge_core::ExecutionStatus::Succeeded,
                started_at: chrono::Utc::now(),
                completed_at: chrono::Utc::now(),
                exit_code: None,
                stdout: "hello runtime".to_string(),
                stderr: String::new(),
                artifacts: vec!["a.txt".to_string()],
                verification: Some(json!({ "fixture": "runtime-executor" })),
                error: None,
            })
        },
    )
}

#[test]
fn full_step_records_executor_result_and_evidence() {
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
        successful_executor(),
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
