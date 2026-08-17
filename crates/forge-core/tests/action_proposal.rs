use forge_core::{
    default_profiles, propose_action, ActionType, AgentRole, MockProvider, PolicyDecision, RiskLevel,
    Task,
};

fn task() -> Task {
    Task {
        id: "task-1".to_string(),
        title: "Inspect repository".to_string(),
        context: "Inspect README.md before making changes".to_string(),
    }
}

fn profile(role: AgentRole) -> forge_core::AgentProfile {
    default_profiles()
        .into_iter()
        .find(|profile| profile.role == role)
        .expect("agent profile")
}

#[test]
fn proposal_generates_typed_action_without_execution() {
    let provider = MockProvider::new(
        serde_json::json!({
            "action": "read_file",
            "reason": "inspect repository",
            "risk": "low",
            "payload": {"path": "README.md"}
        })
        .to_string(),
    );
    let profile = profile(AgentRole::Developer);

    let proposal = propose_action("dev-1", &profile, &provider, &task()).expect("proposal");

    assert_eq!(proposal.action.action_type, ActionType::ReadFile);
    assert_eq!(proposal.action.risk, RiskLevel::Low);
    assert_eq!(proposal.decision, PolicyDecision::Allow);
    assert_eq!(proposal.model, "mock-model");
}

#[test]
fn proposal_preserves_approval_requirement_without_approving() {
    let provider = MockProvider::new(
        serde_json::json!({
            "action": "write_file",
            "reason": "change code",
            "risk": "medium",
            "payload": {"path": "src/lib.rs", "content": "x"}
        })
        .to_string(),
    );
    let profile = profile(AgentRole::Developer);

    let proposal = propose_action("dev-1", &profile, &provider, &task()).expect("proposal");

    assert_eq!(proposal.decision, PolicyDecision::RequireApproval);
    assert!(proposal.action.payload.get("approval_ref").is_none());
    assert!(proposal.action.payload.get("approved").is_none());
}

#[test]
fn proposal_raises_underreported_write_risk_to_trusted_floor() {
    let provider = MockProvider::new(
        serde_json::json!({
            "action": "write_file",
            "reason": "model understates mutation",
            "risk": "low",
            "payload": {"path": "src/lib.rs", "content": "x"}
        })
        .to_string(),
    );
    let profile = profile(AgentRole::Developer);

    let proposal = propose_action("dev-1", &profile, &provider, &task()).expect("proposal");

    assert_eq!(proposal.action.risk, RiskLevel::Medium);
    assert_eq!(proposal.decision, PolicyDecision::RequireApproval);
}

#[test]
fn proposal_raises_underreported_mutating_git_risk_to_trusted_floor() {
    let provider = MockProvider::new(
        serde_json::json!({
            "action": "git",
            "reason": "checkpoint repository",
            "risk": "low",
            "payload": {"operation": "checkpoint", "message": "checkpoint"}
        })
        .to_string(),
    );
    let profile = profile(AgentRole::Developer);

    let proposal = propose_action("dev-1", &profile, &provider, &task()).expect("proposal");

    assert_eq!(proposal.action.risk, RiskLevel::Medium);
    assert_eq!(proposal.decision, PolicyDecision::RequireApproval);
}

#[test]
fn proposal_raises_underreported_rollback_risk_to_high() {
    let provider = MockProvider::new(
        serde_json::json!({
            "action": "git",
            "reason": "rollback repository",
            "risk": "low",
            "payload": {"operation": "rollback", "command": "checkout"}
        })
        .to_string(),
    );
    let profile = profile(AgentRole::Release);

    let proposal = propose_action("release-1", &profile, &provider, &task()).expect("proposal");

    assert_eq!(proposal.action.risk, RiskLevel::High);
    assert_eq!(proposal.decision, PolicyDecision::RequireApproval);
}

#[test]
fn proposal_denies_actions_above_profile_risk_ceiling() {
    let provider = MockProvider::new(
        serde_json::json!({
            "action": "write_file",
            "reason": "unsafe elevation",
            "risk": "high",
            "payload": {"path": "src/lib.rs", "content": "x"}
        })
        .to_string(),
    );
    let profile = profile(AgentRole::Developer);

    let proposal = propose_action("dev-1", &profile, &provider, &task()).expect("proposal");

    assert_eq!(proposal.decision, PolicyDecision::Deny);
    assert!(proposal.action.capabilities.is_empty());
}

#[test]
fn proposal_denies_capability_that_profile_tool_policy_does_not_allow() {
    let provider = MockProvider::new(
        serde_json::json!({
            "action": "write_file",
            "reason": "architect attempts implementation",
            "risk": "low",
            "payload": {"path": "src/lib.rs", "content": "x"}
        })
        .to_string(),
    );
    let profile = profile(AgentRole::Architect);
    assert!(profile
        .capabilities
        .iter()
        .any(|capability| capability.as_str() == "write_file"));
    assert!(!profile.policy.tools.iter().any(|tool| tool == "write_file"));

    let proposal = propose_action("architect-1", &profile, &provider, &task()).expect("proposal");

    assert_eq!(proposal.decision, PolicyDecision::Deny);
    assert!(proposal.action.capabilities.is_empty());
}
