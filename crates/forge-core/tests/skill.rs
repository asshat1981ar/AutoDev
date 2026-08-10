use forge_core::{
    default_skills, route_skills, Capability, DevelopmentContract, RiskLevel, SkillDefinition,
    SkillRegistry,
};

#[test]
fn routing_is_bounded_and_explainable() {
    let registry = default_skills();
    let mut contract = DevelopmentContract::new(
        "Continue development by implementing repository context and verify the feature with tests",
    );
    contract.required_capabilities = vec![Capability::ReadFile, Capability::RunTest];

    let route = route_skills(&registry, &contract, 2);

    assert_eq!(route.selected.len(), 2);
    assert!(route.selected.iter().all(|item| item.score > 0));
    assert!(route.selected.iter().all(|item| !item.reasons.is_empty()));
}

#[test]
fn lower_cost_breaks_equal_score_ties() {
    let mut registry = SkillRegistry::new();
    for (id, cost) in [("expensive", 9), ("cheap", 1)] {
        registry
            .register(SkillDefinition {
                id: id.into(),
                description: id.into(),
                activation_terms: vec!["feature".into()],
                required_capabilities: vec![],
                compatible_roles: vec![],
                risk: RiskLevel::Low,
                verification: vec![],
                cost_hint: cost,
            })
            .unwrap();
    }

    let contract = DevelopmentContract::new("build feature");
    let route = route_skills(&registry, &contract, 1);
    assert_eq!(route.selected[0].skill_id, "cheap");
}

#[test]
fn no_relevance_means_no_skill_is_forced() {
    let registry = default_skills();
    let contract = DevelopmentContract::new("translate a greeting");
    let route = route_skills(&registry, &contract, 5);
    assert!(route.selected.is_empty());
}
