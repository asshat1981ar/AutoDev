use chrono::{Duration, TimeZone, Utc};
use forge_core::{
    evaluate_lease, EffectivePolicy, EvidenceClass, EvidenceRecord, LeaseEvaluationReason,
    LeaseEvaluationStatus, LeasePolicyDefinition, LeasePolicyError, LeasePolicyRegistry, LeaseRule,
    RefreshProposal, RevalidationMode, RiskTier,
};

fn at(hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 17, hour, minute, 0).unwrap()
}

fn evidence() -> EvidenceRecord {
    EvidenceRecord::new(
        "ev-lease",
        "obj-w1",
        "Current architecture claim",
        EvidenceClass::Documented,
        "github",
        "repo://architecture",
        at(7, 0),
        95,
        "stable source content",
        "authoritative source changes",
    )
    .unwrap()
}

fn policy(risk_tier: RiskTier, mode: RevalidationMode) -> EffectivePolicy {
    let mut registry = LeasePolicyRegistry::new();
    registry
        .register(LeasePolicyDefinition {
            id: "architecture-evidence".into(),
            version: "1".into(),
            rules: LeaseRule::AllOf(vec![
                LeaseRule::MaxAge { seconds: 3600 },
                LeaseRule::SourceVersionRequired,
                LeaseRule::ExplicitInvalidationAbsent,
            ]),
            risk_tier,
            revalidation_mode: mode,
        })
        .unwrap();
    registry.compile("architecture-evidence").unwrap()
}

fn proposal(evidence: &EvidenceRecord, policy: &EffectivePolicy) -> RefreshProposal {
    RefreshProposal {
        evidence_id: evidence.id.clone(),
        objective_id: evidence.objective_id.clone(),
        content_fingerprint: evidence.content_fingerprint.clone(),
        source_version: Some("source-v1".into()),
        previous_source_version: Some("source-v1".into()),
        previous_policy_fingerprint: Some(policy.fingerprint.clone()),
        refreshed_at: at(8, 0),
        authoritative: true,
        explicitly_revalidated: true,
        explicitly_invalidated: false,
    }
}

#[test]
fn lease_is_valid_strictly_before_expiry() {
    let evidence = evidence();
    let policy = policy(RiskTier::Medium, RevalidationMode::Explicit);
    let proposal = proposal(&evidence, &policy);

    let evaluation = evaluate_lease(&evidence, &policy, &proposal, at(8, 59)).unwrap();

    assert_eq!(evaluation.status, LeaseEvaluationStatus::Valid);
    assert_eq!(evaluation.reasons, vec![LeaseEvaluationReason::Eligible]);
    assert_eq!(evaluation.valid_until, Some(at(9, 0)));
}

#[test]
fn lease_is_stale_at_exact_expiry() {
    let evidence = evidence();
    let policy = policy(RiskTier::Medium, RevalidationMode::Explicit);
    let proposal = proposal(&evidence, &policy);

    let evaluation = evaluate_lease(&evidence, &policy, &proposal, at(9, 0)).unwrap();

    assert_eq!(evaluation.status, LeaseEvaluationStatus::Rejected);
    assert!(evaluation.reasons.contains(&LeaseEvaluationReason::Expired));
}

#[test]
fn lease_is_stale_after_expiry() {
    let evidence = evidence();
    let policy = policy(RiskTier::Medium, RevalidationMode::Explicit);
    let proposal = proposal(&evidence, &policy);

    let evaluation = evaluate_lease(&evidence, &policy, &proposal, at(9, 1)).unwrap();

    assert_eq!(evaluation.status, LeaseEvaluationStatus::Rejected);
    assert!(evaluation.reasons.contains(&LeaseEvaluationReason::Expired));
}

#[test]
fn source_version_change_requires_review_without_explicit_revalidation() {
    let evidence = evidence();
    let policy = policy(RiskTier::Medium, RevalidationMode::Explicit);
    let mut proposal = proposal(&evidence, &policy);
    proposal.source_version = Some("source-v2".into());
    proposal.explicitly_revalidated = false;

    let evaluation = evaluate_lease(&evidence, &policy, &proposal, at(8, 30)).unwrap();

    assert_eq!(evaluation.status, LeaseEvaluationStatus::ReviewRequired);
    assert!(evaluation
        .reasons
        .contains(&LeaseEvaluationReason::SourceVersionChanged));
}

#[test]
fn fingerprint_change_requires_review_without_explicit_revalidation() {
    let evidence = evidence();
    let policy = policy(RiskTier::Medium, RevalidationMode::Explicit);
    let mut proposal = proposal(&evidence, &policy);
    proposal.content_fingerprint = "a".repeat(64);
    proposal.explicitly_revalidated = false;

    let evaluation = evaluate_lease(&evidence, &policy, &proposal, at(8, 30)).unwrap();

    assert_eq!(evaluation.status, LeaseEvaluationStatus::ReviewRequired);
    assert!(evaluation
        .reasons
        .contains(&LeaseEvaluationReason::FingerprintChanged));
}

#[test]
fn explicit_invalidation_overrides_freshness() {
    let evidence = evidence();
    let policy = policy(RiskTier::Low, RevalidationMode::AutomaticIfUnchanged);
    let mut proposal = proposal(&evidence, &policy);
    proposal.explicitly_revalidated = false;
    proposal.explicitly_invalidated = true;

    let evaluation = evaluate_lease(&evidence, &policy, &proposal, at(8, 10)).unwrap();

    assert_eq!(evaluation.status, LeaseEvaluationStatus::Rejected);
    assert!(evaluation
        .reasons
        .contains(&LeaseEvaluationReason::ExplicitlyInvalidated));
}

#[test]
fn low_risk_unchanged_authoritative_source_can_renew_automatically() {
    let evidence = evidence();
    let policy = policy(RiskTier::Low, RevalidationMode::AutomaticIfUnchanged);
    let mut proposal = proposal(&evidence, &policy);
    proposal.explicitly_revalidated = false;

    let evaluation = evaluate_lease(&evidence, &policy, &proposal, at(8, 30)).unwrap();

    assert_eq!(evaluation.status, LeaseEvaluationStatus::Valid);
    assert_eq!(evaluation.reasons, vec![LeaseEvaluationReason::Eligible]);
}

#[test]
fn medium_and_high_risk_require_explicit_revalidation() {
    for risk in [RiskTier::Medium, RiskTier::High] {
        let evidence = evidence();
        let policy = policy(risk, RevalidationMode::Explicit);
        let mut proposal = proposal(&evidence, &policy);
        proposal.explicitly_revalidated = false;

        let evaluation = evaluate_lease(&evidence, &policy, &proposal, at(8, 30)).unwrap();

        assert_eq!(evaluation.status, LeaseEvaluationStatus::ReviewRequired);
        assert!(evaluation
            .reasons
            .contains(&LeaseEvaluationReason::ExplicitRevalidationRequired));
    }
}

#[test]
fn changed_policy_fingerprint_requires_review_before_renewal() {
    let evidence = evidence();
    let policy = policy(RiskTier::Low, RevalidationMode::AutomaticIfUnchanged);
    let mut proposal = proposal(&evidence, &policy);
    proposal.explicitly_revalidated = false;
    proposal.previous_policy_fingerprint = Some("0".repeat(64));

    let evaluation = evaluate_lease(&evidence, &policy, &proposal, at(8, 30)).unwrap();

    assert_eq!(evaluation.status, LeaseEvaluationStatus::ReviewRequired);
    assert!(evaluation
        .reasons
        .contains(&LeaseEvaluationReason::PolicyChanged));
}

#[test]
fn malformed_refresh_proposal_fails_closed() {
    let evidence = evidence();
    let policy = policy(RiskTier::Low, RevalidationMode::AutomaticIfUnchanged);
    let mut proposal = proposal(&evidence, &policy);
    proposal.content_fingerprint = "broken".into();

    assert_eq!(
        evaluate_lease(&evidence, &policy, &proposal, at(8, 30)).unwrap_err(),
        LeasePolicyError::InvalidRefreshProposal("content_fingerprint"),
    );

    proposal.content_fingerprint = evidence.content_fingerprint.clone();
    proposal.refreshed_at = at(8, 30) + Duration::seconds(1);
    assert_eq!(
        evaluate_lease(&evidence, &policy, &proposal, at(8, 30)).unwrap_err(),
        LeasePolicyError::InvalidRefreshProposal("refreshed_at"),
    );
}
