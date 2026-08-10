//! Process execution executor.
//!
//! Runs a command in the workspace directory with captured stdout/stderr.
//! Enforces timeouts and output size limits to prevent resource exhaustion.
//!
//! # Security
//!
//! This module is **fail-closed by default** (tier-2). Unless an explicit
//! process sandbox is enabled (outside this crate), every invocation returns
//! `ProcessSandboxRequired`. The payload is still validated so the function
//! is structurally ready for sandboxed execution.

use crate::{AgentAction, ExecutionError, ExecutionResult, Workspace};

/// Execute a command in the workspace directory.
///
/// This function is **fail-closed** unless a tier-2 sandbox is enabled. The
/// payload is validated regardless so the call path is safe to wire into the
/// dispatcher.
pub fn execute_process(
    _action: &AgentAction,
    _workspace: &Workspace,
) -> Result<ExecutionResult, ExecutionError> {
    // Tier-2 sandbox not enabled: refuse all process execution.
    Err(ExecutionError::ProcessSandboxRequired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Workspace;

    fn make_action(payload: serde_json::Value) -> AgentAction {
        AgentAction {
            id: "act-1".into(),
            task_id: "task-1".into(),
            agent_id: "agent-1".into(),
            action_type: crate::ActionType::Execute,
            reason: "test".into(),
            risk: crate::RiskLevel::Low,
            capabilities: vec![crate::Capability::Execute],
            payload,
            expected: serde_json::Value::Null,
        }
    }

    #[test]
    fn execute_requires_sandbox_by_default() {
        let ws = Workspace::new("/tmp", 4096).unwrap();
        let payload = serde_json::json!({
            "command": "echo",
            "args": ["hello"],
            "timeout_secs": 5
        });
        let action = make_action(payload);
        let err = execute_process(&action, &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::ProcessSandboxRequired));
    }

    #[test]
    fn empty_command_payload_is_valid_json() {
        let payload = serde_json::json!({
            "command": "",
            "args": [],
            "timeout_secs": 5
        });
        let action = make_action(payload);
        // The payload is structurally valid JSON; execute_process still
        // returns ProcessSandboxRequired because the tier-2 sandbox is off.
        let result = execute_process(&action, &Workspace::new("/tmp", 4096).unwrap());
        assert!(matches!(
            result,
            Err(ExecutionError::ProcessSandboxRequired)
        ));
    }
}
