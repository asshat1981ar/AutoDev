use chrono::{DateTime, Duration, TimeZone, Utc};
use forge_core::{
    attest, evaluate_lease, ArchitectureLeaseError, EffectivePolicy, EvidenceClass, EvidenceRecord,
    LeaseAttestation, LeaseEvaluationStatus, LeasePolicyDefinition, LeaseRule, RefreshProposal,
    RevalidationMode, RiskTier,
};

fn ts(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 18, hour, minute, second)
        .unwrap()
}

fn make_evidence(id: &str, normalized_content: &str) -> EvidenceRecord {
    EvidenceRecord::new(
        id,
        "obj-1",
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

fn policy_with_rule(rule: LeaseRule) -> EffectivePolicy {
    let definition = LeasePolicyDefinition {
        id: "fixture_policy".into(),
        version: "1".into(),
        rule,
        revalidation_mode: RevalidationMode::AutomaticLowRisk,
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

fn policy() -> EffectivePolicy {
    policy_with_rule(LeaseRule::AllOf(vec![
        LeaseRule::MaxAgeSeconds(3600),
        LeaseRule::SourceVersionRequired,
        LeaseRule::FingerprintStable,
        LeaseRule::RiskAtMost(RiskTier::Low),
        LeaseRule::ExplicitInvalidationAbsent,
    ]))
}

fn proposal(previous: &EvidenceRecord) -> RefreshProposal {
    RefreshProposal {
        previous_evidence_id: previous.id.clone(),
        refreshed_evidence: make_evidence("ev-2", "same content"),
        source_version: "v1".into(),
        proposed_at: ts(12, 0, 0),
    }
}

fn prior(
    evidence: &EvidenceRecord,
    policy: &EffectivePolicy,
    valid_until: DateTime<Utc>,
    risk_tier: RiskTier,
) -> LeaseAttestation {
    LeaseAttestation {
        evidence_id: evidence.id.clone(),
        objective_id: evidence.objective_id.clone(),
        evidence_fingerprint: evidence.content_fingerprint.clone(),
        policy_id: policy.id.clone(),
        policy_version: policy.version.clone(),
        policy_fingerprint: policy.policy_fingerprint.clone(),
        source_version: "v1".into(),
        evaluated_at: ts(11, 0, 0),
        valid_until,
        risk_tier,
        attestation_fingerprint: "0".repeat(64),
    }
}

fn valid_automatic_evaluation(
    evidence: &EvidenceRecord,
    policy: &EffectivePolicy,
    proposal: &RefreshProposal,
) -> forge_core::LeaseEvaluation {
    let prior = prior(evidence, policy, ts(12, 0, 0), RiskTier::Low);
    evaluate_lease(
        evidence,
        policy,
        RiskTier::Low,
        ts(12, 0, 1),
        Some(&prior),
        Some(proposal),
        false,
    )
    .unwrap()
}

#[test]
fn valid_low_risk_refresh_issues_bound_attestation_with_derived_expiry() {
    let evidence = make_evidence("ev-1", "same content");
    let policy = policy();
    let proposal = proposal(&evidence);
    let evaluation = valid_automatic_evaluation(&evidence, &policy, &proposal);

    let attestation = attest(&evidence, &policy, &proposal, &evaluation).unwrap();

    assert_eq!(evaluation.status, LeaseEvaluationStatus::Valid);
    assert_eq!(evaluation.risk_tier, RiskTier::Low);
    assert_eq!(attestation.evidence_id, proposal.refreshed_evidence.id);
    assert_eq!(attestation.objective_id, evidence.objective_id);
    assert_eq!(
        attestation.evidence_fingerprint,
        proposal.refreshed_evidence.content_fingerprint
    );
    assert_eq!(attestation.policy_id, policy.id);
    assert_eq!(attestation.policy_version, policy.version);
    assert_eq!(attestation.policy_fingerprint, policy.policy_fingerprint);
    assert_eq!(attestation.source_version, proposal.source_version);
    assert_eq!(attestation.evaluated_at, evaluation.evaluated_at);
    assert_eq!(
        attestation.valid_until,
        evaluation.evaluated_at + Duration::seconds(3600)
    );
    assert_eq!(attestation.risk_tier, RiskTier::Low);
    assert_eq!(attestation.attestation_fingerprint.len(), 64);
    assert!(attestation
        .attestation_fingerprint
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit()));
    attestation.validate().unwrap();
}

#[test]
fn identical_inputs_issue_identical_attestation() {
    let evidence = make_evidence("ev-1", "same content");
    let policy = policy();
    let proposal = proposal(&evidence);
    let evaluation = valid_automatic_evaluation(&evidence, &policy, &proposal);

    let first = attest(&evidence, &policy, &proposal, &evaluation).unwrap();
    let second = attest(&evidence, &policy, &proposal, &evaluation).unwrap();

    assert_eq!(first, second);
}

#[test]
fn non_valid_evaluation_cannot_be_attested() {
    let evidence = make_evidence("ev-1", "same content");
    let policy = policy();
    let proposal = proposal(&evidence);
    let prior = prior(&evidence, &policy, ts(12, 0, 0), RiskTier::High);
    let evaluation = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::High,
        ts(12, 0, 1),
        Some(&prior),
        None,
        false,
    )
    .unwrap();

    assert_eq!(
        attest(&evidence, &policy, &proposal, &evaluation).unwrap_err(),
        ArchitectureLeaseError::EvaluationNotValid(LeaseEvaluationStatus::RevalidationRequired),
    );
}

#[test]
fn issuance_fails_closed_without_unique_max_age() {
    let evidence = make_evidence("ev-1", "same content");
    let no_ttl_policy = policy_with_rule(LeaseRule::AllOf(vec![
        LeaseRule::SourceVersionRequired,
        LeaseRule::FingerprintStable,
        LeaseRule::RiskAtMost(RiskTier::Low),
        LeaseRule::ExplicitInvalidationAbsent,
    ]));
    let proposal = proposal(&evidence);
    let evaluation = valid_automatic_evaluation(&evidence, &no_ttl_policy, &proposal);

    assert_eq!(
        attest(&evidence, &no_ttl_policy, &proposal, &evaluation).unwrap_err(),
        ArchitectureLeaseError::MissingMaxAge,
    );

    let ambiguous_policy = policy_with_rule(LeaseRule::AllOf(vec![
        LeaseRule::MaxAgeSeconds(1800),
        LeaseRule::MaxAgeSeconds(3600),
        LeaseRule::SourceVersionRequired,
        LeaseRule::FingerprintStable,
        LeaseRule::RiskAtMost(RiskTier::Low),
        LeaseRule::ExplicitInvalidationAbsent,
    ]));
    let ambiguous_evaluation = valid_automatic_evaluation(&evidence, &ambiguous_policy, &proposal);

    assert_eq!(
        attest(
            &evidence,
            &ambiguous_policy,
            &proposal,
            &ambiguous_evaluation,
        )
        .unwrap_err(),
        ArchitectureLeaseError::AmbiguousMaxAge,
    );
}

#[test]
fn tampered_attestation_fingerprint_is_rejected() {
    let evidence = make_evidence("ev-1", "same content");
    let policy = policy();
    let proposal = proposal(&evidence);
    let evaluation = valid_automatic_evaluation(&evidence, &policy, &proposal);
    let mut attestation = attest(&evidence, &policy, &proposal, &evaluation).unwrap();

    attestation.source_version = "v2".into();

    assert_eq!(
        attestation.validate().unwrap_err(),
        ArchitectureLeaseError::AttestationFingerprintMismatch,
    );
}

#[test]
fn malformed_binding_fingerprint_is_rejected_before_hash_comparison() {
    let evidence = make_evidence("ev-1", "same content");
    let policy = policy();
    let proposal = proposal(&evidence);
    let evaluation = valid_automatic_evaluation(&evidence, &policy, &proposal);
    let mut attestation = attest(&evidence, &policy, &proposal, &evaluation).unwrap();

    attestation.policy_fingerprint = "not-a-sha256".into();

    assert_eq!(
        attestation.validate().unwrap_err(),
        ArchitectureLeaseError::InvalidAttestationFingerprint("policy_fingerprint"),
    );
}

#[test]
fn non_increasing_attestation_window_is_rejected() {
    let evidence = make_evidence("ev-1", "same content");
    let policy = policy();
    let proposal = proposal(&evidence);
    let evaluation = valid_automatic_evaluation(&evidence, &policy, &proposal);
    let mut attestation = attest(&evidence, &policy, &proposal, &evaluation).unwrap();

    attestation.valid_until = attestation.evaluated_at;

    assert_eq!(
        attestation.validate().unwrap_err(),
        ArchitectureLeaseError::InvalidAttestationWindow,
    );
}
