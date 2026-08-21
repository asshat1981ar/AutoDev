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
    if action.action_type != crate::ActionType::RunTest {
        return Err(ExecutionError::UnsupportedAction(
            action.action_type.as_str().to_string(),
        ));
    }

    enforce_policy(action, grant)?;
    if !has_required_capability(action) {
        return Err(ExecutionError::CapabilityDenied);
    }

    let payload = action
        .payload
        .as_object()
        .ok_or(ExecutionError::PayloadNotObject)?;
    let runner = payload
        .get("runner")
        .ok_or(ExecutionError::MissingPayloadField("runner"))?
        .as_str()
        .ok_or(ExecutionError::PayloadFieldNotString("runner"))?;
    if runner != "cargo" {
        return Err(ExecutionError::UnsafeCommand(runner.to_string()));
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
    fn missing_runner_is_rejected_before_process_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 4096).unwrap();
        let mut action = action(vec![Capability::RunTest]);
        action.payload = json!({"args":["test"]});

        let error =
            run_test_authorized(&action, &workspace, &AuthorizationGrant::none()).unwrap_err();

        assert!(matches!(
            error,
            ExecutionError::MissingPayloadField("runner")
        ));
    }

    #[test]
    fn non_cargo_runner_is_rejected_before_process_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 4096).unwrap();
        let mut action = action(vec![Capability::RunTest]);
        action.payload = json!({"runner":"cargo; rm -rf /","args":[]});

        let error =
            run_test_authorized(&action, &workspace, &AuthorizationGrant::none()).unwrap_err();

        assert!(matches!(error, ExecutionError::UnsafeCommand(_)));
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

    #[test]
    fn non_run_test_action_is_rejected_before_policy_and_process_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 4096).unwrap();
        let mut wrong_action = action(vec![Capability::ReadFile]);
        wrong_action.action_type = ActionType::ReadFile;
        wrong_action.payload = json!({
            "runner": "cargo",
            "args": ["test"],
            "path": "README.md"
        });

        let error = run_test_authorized(&wrong_action, &workspace, &AuthorizationGrant::none())
            .unwrap_err();

        assert!(matches!(
            error,
            ExecutionError::UnsupportedAction(ref action_type) if action_type == "read_file"
        ));
    }
}
