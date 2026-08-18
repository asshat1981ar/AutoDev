use forge_core::{
    ArchitectureLeaseError, LeasePolicyDefinition, LeasePolicyRegistry, LeaseRule,
    RevalidationMode, RiskTier,
};

#[test]
fn built_in_repo_state_policy_resolves_deterministically() {
    let registry = LeasePolicyRegistry::built_ins();

    let first = registry.resolve("repo_state").unwrap();
    let second = registry.resolve("repo_state").unwrap();

    assert_eq!(first, second);
    assert_eq!(first.id, "repo_state");
    assert_eq!(first.version, "1");
    assert_eq!(first.revalidation_mode, RevalidationMode::AutomaticLowRisk);
    assert_eq!(
        first.rule,
        LeaseRule::AllOf(vec![
            LeaseRule::SourceVersionRequired,
            LeaseRule::FingerprintStable,
            LeaseRule::RiskAtMost(RiskTier::Low),
            LeaseRule::ExplicitInvalidationAbsent,
        ]),
    );
    assert_eq!(first.policy_fingerprint.len(), 64);
    assert!(first
        .policy_fingerprint
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn unknown_policy_fails_closed() {
    let registry = LeasePolicyRegistry::built_ins();

    assert!(matches!(
        registry.resolve("missing"),
        Err(ArchitectureLeaseError::UnknownLeasePolicy(id)) if id == "missing"
    ));
}

#[test]
fn zero_max_age_is_rejected() {
    let definition = LeasePolicyDefinition {
        id: "fixture".into(),
        version: "1".into(),
        rule: LeaseRule::MaxAgeSeconds(0),
        revalidation_mode: RevalidationMode::Explicit,
    };

    assert_eq!(
        definition.validate().unwrap_err(),
        ArchitectureLeaseError::InvalidMaxAge(0),
    );
}

#[test]
fn empty_composition_rule_is_rejected() {
    let definition = LeasePolicyDefinition {
        id: "fixture".into(),
        version: "1".into(),
        rule: LeaseRule::AllOf(vec![]),
        revalidation_mode: RevalidationMode::ExplicitOnMaterialChange,
    };

    assert_eq!(
        definition.validate().unwrap_err(),
        ArchitectureLeaseError::EmptyRuleSet("all_of"),
    );
}

#[test]
fn revalidation_modes_remain_closed_and_distinct() {
    assert_ne!(
        RevalidationMode::AutomaticLowRisk,
        RevalidationMode::ExplicitOnMaterialChange,
    );
    assert_ne!(
        RevalidationMode::ExplicitOnMaterialChange,
        RevalidationMode::Explicit,
    );
    assert_ne!(
        RevalidationMode::AutomaticLowRisk,
        RevalidationMode::Explicit,
    );
}
