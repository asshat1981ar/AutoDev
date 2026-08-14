use forge_core::{
    default_profiles, default_skills, plan_dispatch, route_skills, AgentRegistry, AgentRole,
    DevelopmentContract,
};

fn agents() -> AgentRegistry {
    let mut registry = AgentRegistry::new();
    for profile in default_profiles() {
        registry.register(profile);
    }
    registry
}

#[test]
fn forge_loop_routes_then_dispatches_a_vertical_slice() {
    let skills = default_skills();
    let route = route_skills(
        &skills,
        &DevelopmentContract::new("continue development and implement a feature"),
        2,
    );
    let plan = plan_dispatch(&route, &skills, &agents());

    assert!(!plan.assignments.is_empty());
    let build = plan
        .assignments
        .iter()
        .find(|assignment| assignment.skill_id == "build-vertical-slice")
        .expect("build skill should be dispatched");
    assert_eq!(build.agent_role, AgentRole::Developer);
    assert_eq!(build.model.preferred, "qwen2.5-coder");
    assert!(build
        .reasons
        .iter()
        .any(|reason| reason == "selection:least_privilege"));
}

#[test]
fn dispatch_plan_is_serializable_evidence() {
    let skills = default_skills();
    let route = route_skills(
        &skills,
        &DevelopmentContract::new("verify tests and risk boundaries"),
        2,
    );
    let plan = plan_dispatch(&route, &skills, &agents());
    let json = serde_json::to_value(&plan).expect("dispatch plan serializes");

    assert!(json.get("assignments").is_some());
    assert!(json.get("unassigned").is_some());
}
