use chrono::{TimeZone, Utc};
use forge_core::{
    evaluate_lease, EffectivePolicy, EvidenceClass, EvidenceRecord, LeaseAttestation,
    LeaseEvaluationStatus, LeasePolicyDefinition, LeasePolicyError, LeasePolicyRegistry, LeaseRule,
    RefreshProposal, RevalidationMode, RiskTier,
};

fn at(hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 17, hour, minute, 0).unwrap()
}

fn evidence() -> EvidenceRecord {
    EvidenceRecord::new(
        "ev-attest",
        "obj-w1",
        "Attested architecture claim",
        EvidenceClass::Documented,
        "github",
        "repo://attestation",
        at(7, 0),
        95,
        "stable attested content",
        "authoritative source changes",
    )
    .unwrap()
}

fn policy() -> EffectivePolicy {
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
            risk_tier: RiskTier::Medium,
            revalidation_mode: RevalidationMode::Explicit,
        })
        .unwrap();
    registry.compile("architecture-evidence").unwrap()
}

fn valid_evaluation() -> forge_core::LeaseEvaluation {
    let evidence = evidence();
    let policy = policy();
    let proposal = RefreshProposal {
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
    };
    evaluate_lease(&evidence, &policy, &proposal, at(8, 30)).unwrap()
}

#[test]
fn only_valid_evaluations_can_issue_attestations() {
    let mut evaluation = valid_evaluation();
    evaluation.status = LeaseEvaluationStatus::ReviewRequired;

    assert_eq!(
        LeaseAttestation::issue(&evaluation).unwrap_err(),
        LeasePolicyError::AttestationRequiresValidLease,
    );
}

#[test]
fn identical_evaluations_produce_identical_attestation_fingerprints() {
    let evaluation = valid_evaluation();

    let first = LeaseAttestation::issue(&evaluation).unwrap();
    let second = LeaseAttestation::issue(&evaluation).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.attestation_fingerprint.len(), 64);
    first.validate().unwrap();
}

#[test]
fn attestation_binds_exact_evidence_policy_source_time_and_risk_state() {
    let evaluation = valid_evaluation();
    let attestation = LeaseAttestation::issue(&evaluation).unwrap();

    assert_eq!(attestation.evidence_id, evaluation.evidence_id);
    assert_eq!(attestation.objective_id, evaluation.objective_id);
    assert_eq!(
        attestation.evidence_fingerprint,
        evaluation.evidence_fingerprint
    );
    assert_eq!(attestation.policy_id, evaluation.policy_id);
    assert_eq!(attestation.policy_version, evaluation.policy_version);
    assert_eq!(
        attestation.policy_fingerprint,
        evaluation.policy_fingerprint
    );
    assert_eq!(attestation.source_version, "source-v1");
    assert_eq!(attestation.evaluated_at, evaluation.evaluated_at);
    assert_eq!(attestation.valid_until, evaluation.valid_until.unwrap());
    assert_eq!(attestation.risk_tier, evaluation.risk_tier);
}

#[test]
fn tampered_attestation_fails_self_validation() {
    let evaluation = valid_evaluation();
    let mut attestation = LeaseAttestation::issue(&evaluation).unwrap();
    attestation.policy_version = "2".into();

    assert_eq!(
        attestation.validate().unwrap_err(),
        LeasePolicyError::InvalidLeaseAttestationFingerprint,
    );
}
