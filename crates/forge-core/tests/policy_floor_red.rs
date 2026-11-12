//! RED-only regression oracles for PR #52's trusted semantic risk floor.
//! No production policy or execution changes are introduced here.

use forge_core::{
    default_profiles, effective_risk_for_action, enforce_policy, evaluate_policy,
    minimum_risk_for_action, ActionType, AgentAction, AgentRole, AgentRuntime,
    AuthorizationGrant, Capability, MockProvider, PolicyDecision, RiskLevel,
};

fn action(kind: ActionType, operation: Option<&str>) -> AgentAction {
    let payload = match operation {
        Some(value) => serde_json::json!({"operation": value}),
        None => serde_json::json!({}),
    };
    AgentAction {
        id: "security-regression".into(),
        task_id: "task-red".into(),
        agent_id: "untrusted-model".into(),
        action_type: kind,
        reason: "underreported risk must not bypass policy".into(),
        risk: RiskLevel::Low,
        capabilities: vec![Capability::for_action(kind).expect("known action")],
        payload,
        expected: serde_json::Value::Null,
    }
}

#[test]
fn underreported_write_requires_approval_in_canonical_policy() {
    let request = action(ActionType::WriteFile, None);
    assert_eq!(effective_risk_for_action(&request), RiskLevel::Medium);
    assert_eq!(
        evaluate_policy(&request).unwrap(),
        PolicyDecision::RequireApproval,
        "model-reported low risk must not authorize a write"
    );
}

#[test]
fn underreported_execute_requires_approval_in_canonical_policy() {
    let request = action(ActionType::Execute, None);
    assert_eq!(effective_risk_for_action(&request), RiskLevel::High);
    assert_eq!(evaluate_policy(&request).unwrap(), PolicyDecision::RequireApproval);
}

#[test]
fn underreported_checkpoint_cannot_bypass_trusted_grant() {
    let request = action(ActionType::Git, Some("checkpoint"));
    assert_eq!(effective_risk_for_action(&request), RiskLevel::Medium);
    assert!(
        enforce_policy(&request, &AuthorizationGrant::none()).is_err(),
        "mutating git operations must not be allowed without trusted approval"
    );
}

#[test]
fn underreported_rollback_requires_approval() {
    let request = action(ActionType::Git, Some("rollback"));
    assert_eq!(effective_risk_for_action(&request), RiskLevel::High);
    assert_eq!(evaluate_policy(&request).unwrap(), PolicyDecision::RequireApproval);
}

#[test]
fn missing_and_unknown_git_operations_are_not_classified_low() {
    for operation in [None, Some("not-a-real-operation")] {
        let request = action(ActionType::Git, operation);
        assert_eq!(
            minimum_risk_for_action(&request),
            RiskLevel::Critical,
            "unrecognized Git operation should default to a fail-closed classification"
        );
    }
}

#[test]
fn runtime_submission_uses_canonical_policy_decision() {
    let profile = default_profiles()
        .into_iter()
        .find(|profile| profile.role == AgentRole::Developer)
        .expect("developer profile");
    let runtime = AgentRuntime::new(
        "agent-red",
        profile,
        Box::new(MockProvider::new("{}".to_string())),
        Box::new(|_| panic!("policy verification must not execute effects")),
    );
    let request = action(ActionType::WriteFile, None);
    let observed = runtime.submit_to_policy(&request).expect("policy decision");
    assert_eq!(observed, PolicyDecision::RequireApproval);
}

#[test]
fn authorized_read_remains_low_risk() {
    let request = action(ActionType::ReadFile, None);
    assert_eq!(minimum_risk_for_action(&request), RiskLevel::Low);
    assert_eq!(evaluate_policy(&request).unwrap(), PolicyDecision::Allow);
}
