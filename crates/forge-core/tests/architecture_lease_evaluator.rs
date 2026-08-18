use chrono::{DateTime, TimeZone, Utc};
use forge_core::{
    evaluate_lease, ArchitectureLeaseError, EffectivePolicy, EvidenceClass, EvidenceRecord,
    LeaseAttestation, LeaseEvaluationReason, LeaseEvaluationStatus, LeasePolicyDefinition,
    LeaseRule, RefreshProposal, RevalidationMode, RiskTier,
};

fn ts(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 18, hour, minute, second)
        .unwrap()
}

fn evidence(id: &str, objective_id: &str, normalized_content: &str) -> EvidenceRecord {
    EvidenceRecord::new(
        id,
        objective_id,
        "Normalized architecture claim",
        EvidenceClass::RepoObserved,
        "fixture",
        "fixture://source",
        ts(10, 0, 0),
        95,
        normalized_content,
        "source changes materially",
    )
    .unwrap()
}

fn policy(mode: RevalidationMode) -> EffectivePolicy {
    let definition = LeasePolicyDefinition {
        id: "fixture_policy".into(),
        version: "1".into(),
        rule: LeaseRule::AllOf(vec![
            LeaseRule::MaxAgeSeconds(3600),
            LeaseRule::SourceVersionRequired,
            LeaseRule::FingerprintStable,
            LeaseRule::RiskAtMost(RiskTier::High),
            LeaseRule::ExplicitInvalidationAbsent,
        ]),
        revalidation_mode: mode,
    };

    EffectivePolicy {
        id: definition.id.clone(),
        version: definition.version.clone(),
        rule: definition.rule.clone(),
        revalidation_mode: definition.revalidation_mode,
        policy_fingerprint: definition.fingerprint().unwrap(),
        relaxation: None,
    }
}

fn prior_attestation(
    evidence: &EvidenceRecord,
    policy: &EffectivePolicy,
    source_version: &str,
    risk_tier: RiskTier,
    valid_until: DateTime<Utc>,
) -> LeaseAttestation {
    LeaseAttestation {
        evidence_id: evidence.id.clone(),
        objective_id: evidence.objective_id.clone(),
        evidence_fingerprint: evidence.content_fingerprint.clone(),
        policy_id: policy.id.clone(),
        policy_version: policy.version.clone(),
        policy_fingerprint: policy.policy_fingerprint.clone(),
        source_version: source_version.into(),
        evaluated_at: ts(11, 0, 0),
        valid_until,
        risk_tier,
        attestation_fingerprint: "0".repeat(64),
    }
}

fn refresh(
    previous: &EvidenceRecord,
    refreshed: EvidenceRecord,
    source_version: &str,
) -> RefreshProposal {
    RefreshProposal {
        previous_evidence_id: previous.id.clone(),
        refreshed_evidence: refreshed,
        source_version: source_version.into(),
        proposed_at: ts(12, 0, 0),
    }
}

#[test]
fn fresh_prior_attestation_is_valid_before_expiry() {
    let evidence = evidence("ev-1", "obj-1", "same content");
    let policy = policy(RevalidationMode::Explicit);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Low, ts(12, 0, 0));

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::Low,
        ts(11, 59, 59),
        Some(&prior),
        None,
        false,
    )
    .unwrap();

    assert_eq!(result.status, LeaseEvaluationStatus::Valid);
    assert_eq!(result.reason, LeaseEvaluationReason::FreshWithinPolicy);
}

#[test]
fn attestation_is_stale_at_exact_valid_until_boundary() {
    let evidence = evidence("ev-1", "obj-1", "same content");
    let policy = policy(RevalidationMode::Explicit);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Low, ts(12, 0, 0));

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::Low,
        ts(12, 0, 0),
        Some(&prior),
        None,
        false,
    )
    .unwrap();

    assert_eq!(result.status, LeaseEvaluationStatus::Stale);
    assert_eq!(result.reason, LeaseEvaluationReason::TtlExpired);
}

#[test]
fn high_risk_expired_evidence_requires_explicit_revalidation() {
    let evidence = evidence("ev-1", "obj-1", "same content");
    let policy = policy(RevalidationMode::Explicit);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::High, ts(12, 0, 0));

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::High,
        ts(12, 0, 1),
        Some(&prior),
        None,
        false,
    )
    .unwrap();

    assert_eq!(result.status, LeaseEvaluationStatus::RevalidationRequired);
    assert_eq!(
        result.reason,
        LeaseEvaluationReason::HighRiskReviewRequired
    );
}

#[test]
fn unchanged_low_risk_refresh_can_renew_automatically_after_expiry() {
    let evidence = evidence("ev-1", "obj-1", "same content");
    let policy = policy(RevalidationMode::AutomaticLowRisk);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Low, ts(12, 0, 0));
    let proposal = refresh(
        &evidence,
        evidence("ev-2", "obj-1", "same content"),
        "v1",
    );

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::Low,
        ts(12, 0, 1),
        Some(&prior),
        Some(&proposal),
        false,
    )
    .unwrap();

    assert_eq!(result.status, LeaseEvaluationStatus::Valid);
    assert_eq!(result.reason, LeaseEvaluationReason::FreshWithinPolicy);
}

#[test]
fn changed_source_version_requires_revalidation() {
    let evidence = evidence("ev-1", "obj-1", "same content");
    let policy = policy(RevalidationMode::AutomaticLowRisk);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Low, ts(13, 0, 0));
    let proposal = refresh(
        &evidence,
        evidence("ev-2", "obj-1", "same content"),
        "v2",
    );

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::Low,
        ts(12, 0, 0),
        Some(&prior),
        Some(&proposal),
        false,
    )
    .unwrap();

    assert_eq!(result.status, LeaseEvaluationStatus::RevalidationRequired);
    assert_eq!(result.reason, LeaseEvaluationReason::SourceVersionChanged);
}

#[test]
fn changed_fingerprint_under_same_source_version_requires_revalidation() {
    let evidence = evidence("ev-1", "obj-1", "original content");
    let policy = policy(RevalidationMode::AutomaticLowRisk);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Low, ts(13, 0, 0));
    let proposal = refresh(
        &evidence,
        evidence("ev-2", "obj-1", "changed content"),
        "v1",
    );

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::Low,
        ts(12, 0, 0),
        Some(&prior),
        Some(&proposal),
        false,
    )
    .unwrap();

    assert_eq!(result.status, LeaseEvaluationStatus::RevalidationRequired);
    assert_eq!(result.reason, LeaseEvaluationReason::FingerprintChanged);
}

#[test]
fn medium_risk_material_change_requires_review() {
    let evidence = evidence("ev-1", "obj-1", "original content");
    let policy = policy(RevalidationMode::ExplicitOnMaterialChange);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Medium, ts(13, 0, 0));
    let proposal = refresh(
        &evidence,
        evidence("ev-2", "obj-1", "changed content"),
        "v1",
    );

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::Medium,
        ts(12, 0, 0),
        Some(&prior),
        Some(&proposal),
        false,
    )
    .unwrap();

    assert_eq!(result.status, LeaseEvaluationStatus::RevalidationRequired);
    assert_eq!(
        result.reason,
        LeaseEvaluationReason::MediumRiskReviewRequired
    );
}

#[test]
fn explicit_invalidation_overrides_fresh_ttl() {
    let evidence = evidence("ev-1", "obj-1", "same content");
    let policy = policy(RevalidationMode::Explicit);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Low, ts(13, 0, 0));

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::Low,
        ts(12, 0, 0),
        Some(&prior),
        None,
        true,
    )
    .unwrap();

    assert_eq!(result.status, LeaseEvaluationStatus::Invalid);
    assert_eq!(result.reason, LeaseEvaluationReason::ExplicitlyInvalidated);
}

#[test]
fn changed_policy_fingerprint_requires_revalidation() {
    let evidence = evidence("ev-1", "obj-1", "same content");
    let policy = policy(RevalidationMode::Explicit);
    let mut prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Low, ts(13, 0, 0));
    prior.policy_fingerprint = "f".repeat(64);

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::Low,
        ts(12, 0, 0),
        Some(&prior),
        None,
        false,
    )
    .unwrap();

    assert_eq!(result.status, LeaseEvaluationStatus::RevalidationRequired);
    assert_eq!(result.reason, LeaseEvaluationReason::PolicyChanged);
}

#[test]
fn malformed_refresh_proposal_fails_closed() {
    let evidence = evidence("ev-1", "obj-1", "same content");
    let policy = policy(RevalidationMode::AutomaticLowRisk);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Low, ts(13, 0, 0));
    let proposal = RefreshProposal {
        previous_evidence_id: "".into(),
        refreshed_evidence: evidence("ev-2", "obj-1", "same content"),
        source_version: "v1".into(),
        proposed_at: ts(12, 0, 0),
    };

    assert_eq!(
        evaluate_lease(
            &evidence,
            &policy,
            RiskTier::Low,
            ts(12, 0, 0),
            Some(&prior),
            Some(&proposal),
            false,
        )
        .unwrap_err(),
        ArchitectureLeaseError::EmptyField("previous_evidence_id"),
    );
}

#[test]
fn refresh_proposal_cannot_cross_objective_boundary() {
    let evidence = evidence("ev-1", "obj-1", "same content");
    let policy = policy(RevalidationMode::AutomaticLowRisk);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Low, ts(13, 0, 0));
    let proposal = refresh(
        &evidence,
        evidence("ev-2", "obj-other", "same content"),
        "v1",
    );

    assert_eq!(
        evaluate_lease(
            &evidence,
            &policy,
            RiskTier::Low,
            ts(12, 0, 0),
            Some(&prior),
            Some(&proposal),
            false,
        )
        .unwrap_err(),
        ArchitectureLeaseError::RefreshObjectiveMismatch {
            expected: "obj-1".into(),
            actual: "obj-other".into(),
        },
    );
}

#[test]
fn refresh_proposal_cannot_overwrite_previous_evidence_record() {
    let evidence = evidence("ev-1", "obj-1", "same content");
    let policy = policy(RevalidationMode::AutomaticLowRisk);
    let prior = prior_attestation(&evidence, &policy, "v1", RiskTier::Low, ts(13, 0, 0));
    let proposal = refresh(
        &evidence,
        evidence("ev-1", "obj-1", "same content"),
        "v1",
    );

    assert_eq!(
        evaluate_lease(
            &evidence,
            &policy,
            RiskTier::Low,
            ts(12, 0, 0),
            Some(&prior),
            Some(&proposal),
            false,
        )
        .unwrap_err(),
        ArchitectureLeaseError::RefreshOverwritesPreviousEvidence("ev-1".into()),
    );
}
