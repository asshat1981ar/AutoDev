use forge_core::{
    ApprovalReference, ApprovalReferenceKind, ArchitectureLeaseError, LeasePolicyDefinition,
    LeasePolicyRegistry, LeaseRule, PolicyRelaxation, RepositoryApprovalEvidence,
    RepositoryPolicyOverride, RevalidationMode, RiskTier,
};

fn repo_state_rule(risk: RiskTier) -> LeaseRule {
    LeaseRule::AllOf(vec![
        LeaseRule::SourceVersionRequired,
        LeaseRule::FingerprintStable,
        LeaseRule::RiskAtMost(risk),
        LeaseRule::ExplicitInvalidationAbsent,
    ])
}

fn approval_reference() -> ApprovalReference {
    ApprovalReference {
        repository: "asshat1981ar/AutoDev".into(),
        kind: ApprovalReferenceKind::Commit,
        reference: "abc123".into(),
    }
}

fn relaxed_repo_state_definition() -> LeasePolicyDefinition {
    LeasePolicyDefinition {
        id: "repo_state".into(),
        version: "2".into(),
        rule: repo_state_rule(RiskTier::Medium),
        revalidation_mode: RevalidationMode::AutomaticLowRisk,
    }
}

#[test]
fn built_in_repo_state_policy_resolves_deterministically() {
    let registry = LeasePolicyRegistry::built_ins();

    let first = registry.resolve("repo_state").unwrap();
    let second = registry.resolve("repo_state").unwrap();

    assert_eq!(first, second);
    assert_eq!(first.id, "repo_state");
    assert_eq!(first.version, "1");
    assert_eq!(first.revalidation_mode, RevalidationMode::AutomaticLowRisk);
    assert_eq!(first.rule, repo_state_rule(RiskTier::Low));
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

#[test]
fn repository_policy_may_tighten_builtin_without_approval() {
    let registry = LeasePolicyRegistry::built_ins();
    let repository_override = RepositoryPolicyOverride {
        definition: LeasePolicyDefinition {
            id: "repo_state".into(),
            version: "2".into(),
            rule: repo_state_rule(RiskTier::Low),
            revalidation_mode: RevalidationMode::Explicit,
        },
        relaxation: None,
    };

    let effective = registry
        .compile("repo_state", Some(&repository_override), None)
        .unwrap();

    assert_eq!(effective.version, "2");
    assert_eq!(effective.revalidation_mode, RevalidationMode::Explicit);
    assert_eq!(effective.relaxation, None);
}

#[test]
fn repository_policy_cannot_relax_builtin_silently() {
    let registry = LeasePolicyRegistry::built_ins();
    let repository_override = RepositoryPolicyOverride {
        definition: relaxed_repo_state_definition(),
        relaxation: None,
    };

    assert_eq!(
        registry
            .compile("repo_state", Some(&repository_override), None)
            .unwrap_err(),
        ArchitectureLeaseError::UnsafePolicyRelaxation,
    );
}

#[test]
fn relaxation_requires_repository_backed_approval_evidence() {
    let registry = LeasePolicyRegistry::built_ins();
    let repository_override = RepositoryPolicyOverride {
        definition: relaxed_repo_state_definition(),
        relaxation: Some(PolicyRelaxation {
            allow_relaxation: true,
            rationale: "Approved compatibility exception".into(),
            approval_reference: approval_reference(),
        }),
    };

    assert_eq!(
        registry
            .compile("repo_state", Some(&repository_override), None)
            .unwrap_err(),
        ArchitectureLeaseError::RelaxationApprovalRequired,
    );
}

#[test]
fn relaxation_rejects_mismatched_approval_evidence() {
    let registry = LeasePolicyRegistry::built_ins();
    let definition = relaxed_repo_state_definition();
    let repository_override = RepositoryPolicyOverride {
        definition: definition.clone(),
        relaxation: Some(PolicyRelaxation {
            allow_relaxation: true,
            rationale: "Approved compatibility exception".into(),
            approval_reference: approval_reference(),
        }),
    };
    let approval = RepositoryApprovalEvidence {
        approval_reference: approval_reference(),
        policy_id: "repo_state".into(),
        approved_policy_version: "2".into(),
        approved_policy_fingerprint: "0".repeat(64),
    };

    assert_eq!(
        registry
            .compile("repo_state", Some(&repository_override), Some(&approval))
            .unwrap_err(),
        ArchitectureLeaseError::ApprovalEvidenceMismatch,
    );
}

#[test]
fn matching_commit_approval_allows_explicit_relaxation() {
    let registry = LeasePolicyRegistry::built_ins();
    let definition = relaxed_repo_state_definition();
    let candidate_fingerprint = definition.fingerprint().unwrap();
    let relaxation = PolicyRelaxation {
        allow_relaxation: true,
        rationale: "Approved compatibility exception".into(),
        approval_reference: approval_reference(),
    };
    let repository_override = RepositoryPolicyOverride {
        definition: definition.clone(),
        relaxation: Some(relaxation.clone()),
    };
    let approval = RepositoryApprovalEvidence {
        approval_reference: approval_reference(),
        policy_id: definition.id.clone(),
        approved_policy_version: definition.version.clone(),
        approved_policy_fingerprint: candidate_fingerprint.clone(),
    };

    let effective = registry
        .compile("repo_state", Some(&repository_override), Some(&approval))
        .unwrap();

    assert_eq!(effective.policy_fingerprint, candidate_fingerprint);
    assert_eq!(effective.relaxation, Some(relaxation));
}

#[test]
fn malformed_approval_fingerprint_fails_closed() {
    let approval = RepositoryApprovalEvidence {
        approval_reference: approval_reference(),
        policy_id: "repo_state".into(),
        approved_policy_version: "2".into(),
        approved_policy_fingerprint: "not-a-sha256".into(),
    };

    assert_eq!(
        approval.validate().unwrap_err(),
        ArchitectureLeaseError::InvalidApprovalFingerprint("not-a-sha256".into()),
    );
}
