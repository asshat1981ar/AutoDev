use forge_core::{
    default_profiles, propose_action, ActionType, AgentRole, MockProvider, PolicyDecision, Task,
};

fn task() -> Task {
    Task {
        id: "task-1".to_string(),
        title: "Inspect repository".to_string(),
        context: "Inspect README.md before making changes".to_string(),
    }
}

fn developer_profile() -> forge_core::AgentProfile {
    default_profiles()
        .into_iter()
        .find(|profile| profile.role == AgentRole::Developer)
        .expect("developer profile")
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
    let profile = developer_profile();

    let proposal = propose_action("dev-1", &profile, &provider, &task()).expect("proposal");

    assert_eq!(proposal.action.action_type, ActionType::ReadFile);
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
    let profile = developer_profile();

    let proposal = propose_action("dev-1", &profile, &provider, &task()).expect("proposal");

    assert_eq!(proposal.decision, PolicyDecision::RequireApproval);
    assert!(proposal.action.payload.get("approval_ref").is_none());
    assert!(proposal.action.payload.get("approved").is_none());
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
    let profile = developer_profile();

    let proposal = propose_action("dev-1", &profile, &provider, &task()).expect("proposal");

    assert_eq!(proposal.decision, PolicyDecision::Deny);
    assert!(proposal.action.capabilities.is_empty());
}
