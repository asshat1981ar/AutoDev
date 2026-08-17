use forge_core::{
    ApprovalReference, ApprovalReferenceKind, LeasePolicyDefinition, LeasePolicyError,
    LeasePolicyRegistry, LeaseRule, PolicyRelaxation, RepositoryApprovalEvidence,
    RepositoryPolicyOverride, RevalidationMode, RiskTier,
};

fn max_age_policy(seconds: u64) -> LeasePolicyDefinition {
    LeasePolicyDefinition {
        id: "architecture-evidence".into(),
        version: "1".into(),
        rules: LeaseRule::MaxAge { seconds },
        risk_tier: RiskTier::Medium,
        revalidation_mode: RevalidationMode::Explicit,
    }
}

fn approval(reference: &str) -> ApprovalReference {
    ApprovalReference {
        kind: ApprovalReferenceKind::Commit,
        reference: reference.into(),
    }
}

#[test]
fn repository_override_can_tighten_without_relaxation_approval() {
    let mut registry = LeasePolicyRegistry::new();
    registry.register(max_age_policy(3600)).unwrap();
    let override_policy = RepositoryPolicyOverride {
        policy_id: "architecture-evidence".into(),
        replacement: max_age_policy(1800),
        allow_relaxation: false,
        relaxation: None,
    };

    let effective = registry
        .compile_with_override("architecture-evidence", &override_policy, &[])
        .unwrap();

    assert_eq!(effective.rules, LeaseRule::MaxAge { seconds: 1800 });
}

#[test]
fn repository_override_rejects_relaxation_when_not_allowed() {
    let mut registry = LeasePolicyRegistry::new();
    registry.register(max_age_policy(3600)).unwrap();
    let override_policy = RepositoryPolicyOverride {
        policy_id: "architecture-evidence".into(),
        replacement: max_age_policy(7200),
        allow_relaxation: false,
        relaxation: None,
    };

    assert_eq!(
        registry
            .compile_with_override("architecture-evidence", &override_policy, &[])
            .unwrap_err(),
        LeasePolicyError::RelaxationNotAllowed,
    );
}

#[test]
fn repository_override_requires_rationale_and_matching_repository_approval() {
    let mut registry = LeasePolicyRegistry::new();
    registry.register(max_age_policy(3600)).unwrap();
    let approved_ref = approval("abc123");
    let override_policy = RepositoryPolicyOverride {
        policy_id: "architecture-evidence".into(),
        replacement: max_age_policy(7200),
        allow_relaxation: true,
        relaxation: Some(PolicyRelaxation {
            rationale: "temporary compatibility window".into(),
            approval_reference: approved_ref.clone(),
        }),
    };

    assert_eq!(
        registry
            .compile_with_override("architecture-evidence", &override_policy, &[])
            .unwrap_err(),
        LeasePolicyError::MissingRepositoryApproval(approved_ref.clone()),
    );

    let mismatched = RepositoryApprovalEvidence {
        reference: approval("different"),
        approved: true,
    };
    assert_eq!(
        registry
            .compile_with_override("architecture-evidence", &override_policy, &[mismatched])
            .unwrap_err(),
        LeasePolicyError::MissingRepositoryApproval(approved_ref.clone()),
    );

    let matching = RepositoryApprovalEvidence {
        reference: approved_ref,
        approved: true,
    };
    let effective = registry
        .compile_with_override("architecture-evidence", &override_policy, &[matching])
        .unwrap();
    assert_eq!(effective.rules, LeaseRule::MaxAge { seconds: 7200 });
}

#[test]
fn unsupported_policy_comparison_fails_closed() {
    let mut registry = LeasePolicyRegistry::new();
    registry.register(max_age_policy(3600)).unwrap();
    let mut replacement = max_age_policy(3600);
    replacement.rules = LeaseRule::AnyOf(vec![
        LeaseRule::FingerprintStable,
        LeaseRule::SourceVersionRequired,
    ]);
    let override_policy = RepositoryPolicyOverride {
        policy_id: "architecture-evidence".into(),
        replacement,
        allow_relaxation: true,
        relaxation: Some(PolicyRelaxation {
            rationale: "unsupported comparison fixture".into(),
            approval_reference: approval("abc123"),
        }),
    };

    assert_eq!(
        registry
            .compile_with_override("architecture-evidence", &override_policy, &[])
            .unwrap_err(),
        LeasePolicyError::UnsupportedPolicyComparison,
    );
}
