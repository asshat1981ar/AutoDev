use chrono::{DateTime, TimeZone, Utc};
use forge_core::{
    evaluate_lease, EffectivePolicy, EvidenceClass, EvidenceRecord, LeaseAttestation,
    LeaseEvaluationStatus, LeasePolicyDefinition, LeaseRule, RevalidationMode, RiskTier,
};

fn ts(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 18, hour, minute, second)
        .unwrap()
}

fn evidence() -> EvidenceRecord {
    EvidenceRecord::new(
        "ev-1",
        "obj-1",
        "Normalized architecture claim",
        EvidenceClass::RepoObserved,
        "fixture",
        "fixture://source",
        ts(10, 0, 0),
        95,
        "same content",
        "source changes materially",
    )
    .unwrap()
}

fn policy() -> EffectivePolicy {
    let definition = LeasePolicyDefinition {
        id: "fixture_policy".into(),
        version: "1".into(),
        rule: LeaseRule::AllOf(vec![
            LeaseRule::MaxAgeSeconds(3600),
            LeaseRule::SourceVersionRequired,
            LeaseRule::FingerprintStable,
            LeaseRule::RiskAtMost(RiskTier::Low),
            LeaseRule::ExplicitInvalidationAbsent,
        ]),
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

#[test]
fn malformed_prior_attestation_cannot_make_evidence_current() {
    let evidence = evidence();
    let policy = policy();
    let forged_prior = LeaseAttestation {
        evidence_id: evidence.id.clone(),
        objective_id: evidence.objective_id.clone(),
        evidence_fingerprint: evidence.content_fingerprint.clone(),
        policy_id: policy.id.clone(),
        policy_version: policy.version.clone(),
        policy_fingerprint: policy.policy_fingerprint.clone(),
        source_version: "v1".into(),
        evaluated_at: ts(11, 0, 0),
        valid_until: ts(13, 0, 0),
        risk_tier: RiskTier::Low,
        // Correct shape but intentionally not the deterministic fingerprint for these fields.
        attestation_fingerprint: "0".repeat(64),
    };

    assert!(forged_prior.validate().is_err());

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::Low,
        ts(12, 0, 0),
        Some(&forged_prior),
        None,
        false,
    );

    assert!(
        result.is_err(),
        "malformed prior attestations must fail closed instead of yielding {:?}",
        result.map(|evaluation| evaluation.status)
    );
}

#[test]
fn forged_prior_currently_demonstrates_the_bypass_if_not_rejected() {
    let evidence = evidence();
    let policy = policy();
    let forged_prior = LeaseAttestation {
        evidence_id: evidence.id.clone(),
        objective_id: evidence.objective_id.clone(),
        evidence_fingerprint: evidence.content_fingerprint.clone(),
        policy_id: policy.id.clone(),
        policy_version: policy.version.clone(),
        policy_fingerprint: policy.policy_fingerprint.clone(),
        source_version: "v1".into(),
        evaluated_at: ts(11, 0, 0),
        valid_until: ts(13, 0, 0),
        risk_tier: RiskTier::Low,
        attestation_fingerprint: "0".repeat(64),
    };

    let result = evaluate_lease(
        &evidence,
        &policy,
        RiskTier::Low,
        ts(12, 0, 0),
        Some(&forged_prior),
        None,
        false,
    );

    if let Ok(evaluation) = result {
        assert_ne!(
            evaluation.status,
            LeaseEvaluationStatus::Valid,
            "an invalid prior attestation must never make current evidence eligible"
        );
    }
}
