use forge_core::{
    LeasePolicyDefinition, LeasePolicyError, LeasePolicyRegistry, LeaseRule, RevalidationMode,
    RiskTier,
};

fn policy(id: &str, rules: LeaseRule) -> LeasePolicyDefinition {
    LeasePolicyDefinition {
        id: id.into(),
        version: "1".into(),
        rules,
        risk_tier: RiskTier::Medium,
        revalidation_mode: RevalidationMode::Explicit,
    }
}

#[test]
fn policy_fingerprint_is_deterministic_for_equivalent_rule_order() {
    let left = policy(
        "architecture-evidence",
        LeaseRule::AllOf(vec![
            LeaseRule::FingerprintStable,
            LeaseRule::SourceVersionRequired,
            LeaseRule::MaxAge { seconds: 3600 },
        ]),
    );
    let right = policy(
        "architecture-evidence",
        LeaseRule::AllOf(vec![
            LeaseRule::MaxAge { seconds: 3600 },
            LeaseRule::SourceVersionRequired,
            LeaseRule::FingerprintStable,
        ]),
    );

    assert_eq!(left.fingerprint().unwrap(), right.fingerprint().unwrap());
    assert_eq!(left.fingerprint().unwrap().len(), 64);
}

#[test]
fn policy_validation_rejects_empty_composite_rules() {
    let definition = policy("architecture-evidence", LeaseRule::AllOf(vec![]));

    assert_eq!(
        definition.validate().unwrap_err(),
        LeasePolicyError::EmptyCompositeRule("all_of"),
    );
}

#[test]
fn policy_registry_rejects_duplicate_policy_ids() {
    let mut registry = LeasePolicyRegistry::new();
    registry
        .register(policy(
            "architecture-evidence",
            LeaseRule::ExplicitInvalidationAbsent,
        ))
        .unwrap();

    assert_eq!(
        registry
            .register(policy(
                "architecture-evidence",
                LeaseRule::SourceVersionRequired,
            ))
            .unwrap_err(),
        LeasePolicyError::DuplicatePolicyId("architecture-evidence".into()),
    );
}

#[test]
fn registry_compiles_a_valid_policy_with_bound_fingerprint() {
    let mut registry = LeasePolicyRegistry::new();
    registry
        .register(policy(
            "architecture-evidence",
            LeaseRule::AllOf(vec![
                LeaseRule::ExplicitInvalidationAbsent,
                LeaseRule::RiskAtMost(RiskTier::High),
            ]),
        ))
        .unwrap();

    let effective = registry.compile("architecture-evidence").unwrap();

    assert_eq!(effective.id, "architecture-evidence");
    assert_eq!(effective.version, "1");
    assert_eq!(effective.risk_tier, RiskTier::Medium);
    assert_eq!(effective.revalidation_mode, RevalidationMode::Explicit);
    assert_eq!(effective.fingerprint.len(), 64);
}

#[test]
fn registry_fails_closed_for_unknown_policy() {
    let registry = LeasePolicyRegistry::new();

    assert_eq!(
        registry.compile("missing").unwrap_err(),
        LeasePolicyError::UnknownPolicyId("missing".into()),
    );
}
