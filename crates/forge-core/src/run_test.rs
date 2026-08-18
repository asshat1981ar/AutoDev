//! Typed test-execution adapter.
//!
//! RunTest is recognized as a first-class ForgeCore action, but process
//! execution remains fail-closed until the tier-2 sandbox is available.

use crate::{
    enforce_policy, has_required_capability, AgentAction, AuthorizationGrant, ExecutionError,
    ExecutionResult, Workspace,
};

pub fn run_test_authorized(
    action: &AgentAction,
    workspace: &Workspace,
    grant: &AuthorizationGrant,
) -> Result<ExecutionResult, ExecutionError> {
    enforce_policy(action, grant)?;
    if !has_required_capability(action) {
        return Err(ExecutionError::CapabilityDenied);
    }

    crate::execute_process(action, workspace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActionType, Capability, RiskLevel};
    use serde_json::json;

    fn action(capabilities: Vec<Capability>) -> AgentAction {
        AgentAction {
            id: "run-test-1".into(),
            task_id: "task-1".into(),
            agent_id: "tester-1".into(),
            action_type: ActionType::RunTest,
            reason: "verify repository tests".into(),
            risk: RiskLevel::Low,
            capabilities,
            payload: json!({"runner":"cargo","args":["test"]}),
            expected: json!({}),
        }
    }

    #[test]
    fn missing_run_test_capability_is_denied_before_process_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 4096).unwrap();
        let error = run_test_authorized(&action(vec![]), &workspace, &AuthorizationGrant::none())
            .unwrap_err();
        assert!(matches!(error, ExecutionError::CapabilityDenied));
    }

    #[test]
    fn authorized_run_test_fails_closed_without_tier_two_sandbox() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 4096).unwrap();
        let error = run_test_authorized(
            &action(vec![Capability::RunTest]),
            &workspace,
            &AuthorizationGrant::none(),
        )
        .unwrap_err();
        assert!(matches!(error, ExecutionError::ProcessSandboxRequired));
    }
}
