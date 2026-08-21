use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use forge_core::{
    attest, evaluate_current_verification, evaluate_lease, ArchitectureAlternative,
    ArchitectureDecision, CurrentVerificationStatus, DecisionMaturity, EffectivePolicy,
    EvidenceClass, EvidenceRecord, LeaseAttestation, LeasePolicyDefinition, LeaseRule,
    RefreshProposal, RevalidationMode, Reversibility, RiskTier,
};

fn ts(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 18, hour, minute, second)
        .unwrap()
}

fn evidence(id: &str, class: EvidenceClass, content: &str) -> EvidenceRecord {
    EvidenceRecord::new(
        id,
        "obj-current",
        "Current architecture claim",
        class,
        "fixture",
        "fixture://source",
        ts(10, 0, 0),
        95,
        content,
        "source changes materially",
    )
    .unwrap()
}

fn policy(version: &str, max_age_seconds: u64) -> EffectivePolicy {
    let definition = LeasePolicyDefinition {
        id: "current_policy".into(),
        version: version.into(),
        rule: LeaseRule::AllOf(vec![
            LeaseRule::MaxAgeSeconds(max_age_seconds),
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

fn decision(evidence_id: &str, maturity: DecisionMaturity) -> ArchitectureDecision {
    ArchitectureDecision {
        id: "dec-current".into(),
        objective_id: "obj-current".into(),
        decision: "Use current evidence only".into(),
        alternatives: vec![ArchitectureAlternative {
            name: "reuse indefinitely".into(),
            rationale: "rejected because freshness matters".into(),
            rejected: true,
        }],
        contradiction: "reuse vs freshness".into(),
        selected_option: "lease-aware verification".into(),
        rationale: "separate historical truth from current eligibility".into(),
        evidence_refs: vec![evidence_id.into()],
        reversibility: Reversibility::Easy,
        risks: vec!["stale evidence".into()],
        invalidation_conditions: vec!["source changes".into()],
        maturity,
    }
}

fn current_fixture(class: EvidenceClass) -> (EvidenceRecord, EffectivePolicy, LeaseAttestation) {
    let previous = evidence("ev-previous", class, "same content");
    let current = evidence("ev-current", class, "same content");
    let policy = policy("1", 3600);
    let proposal = RefreshProposal {
        previous_evidence_id: previous.id.clone(),
        refreshed_evidence: current.clone(),
        source_version: "v1".into(),
        proposed_at: ts(11, 0, 0),
    };
    let prior = LeaseAttestation {
        evidence_id: previous.id.clone(),
        objective_id: previous.objective_id.clone(),
        evidence_fingerprint: previous.content_fingerprint.clone(),
        policy_id: policy.id.clone(),
        policy_version: policy.version.clone(),
        policy_fingerprint: policy.policy_fingerprint.clone(),
        source_version: "v1".into(),
        evaluated_at: ts(10, 0, 0),
        valid_until: ts(11, 0, 0),
        risk_tier: RiskTier::Low,
        attestation_fingerprint: "0".repeat(64),
    };
    let evaluation = evaluate_lease(
        &previous,
        &policy,
        RiskTier::Low,
        ts(11, 0, 1),
        Some(&prior),
        Some(&proposal),
        false,
    )
    .unwrap();
    let attestation = attest(&previous, &policy, &proposal, &evaluation).unwrap();
    (current, policy, attestation)
}

fn maps(
    record: EvidenceRecord,
    policy: EffectivePolicy,
    attestation: LeaseAttestation,
) -> (
    BTreeMap<String, EvidenceRecord>,
    BTreeMap<String, LeaseAttestation>,
    BTreeMap<String, EffectivePolicy>,
) {
    let evidence = BTreeMap::from([(record.id.clone(), record)]);
    let attestations = BTreeMap::from([(attestation.evidence_id.clone(), attestation)]);
    let policies = BTreeMap::from([(policy.id.clone(), policy)]);
    (evidence, attestations, policies)
}

#[test]
fn supported_evidence_with_exact_current_attestation_is_eligible() {
    let (record, policy, attestation) = current_fixture(EvidenceClass::RepoObserved);
    let decision = decision(&record.id, DecisionMaturity::Verified);
    let (evidence, attestations, policies) = maps(record, policy, attestation);

    let result = evaluate_current_verification(
        &decision,
        &evidence,
        &attestations,
        &policies,
        ts(11, 30, 0),
    )
    .unwrap();

    assert_eq!(result.status, CurrentVerificationStatus::Eligible);
    assert_eq!(result.eligible_evidence_ids, vec!["ev-current"]);
    assert!(result.ineligible_evidence_ids.is_empty());
}

#[test]
fn expired_attestation_is_ineligible_without_rewriting_historical_maturity() {
    let (record, policy, attestation) = current_fixture(EvidenceClass::RepoObserved);
    let decision = decision(&record.id, DecisionMaturity::Verified);
    let (evidence, attestations, policies) = maps(record, policy, attestation);

    let result =
        evaluate_current_verification(&decision, &evidence, &attestations, &policies, ts(12, 0, 2))
            .unwrap();

    assert_eq!(result.status, CurrentVerificationStatus::Ineligible);
    assert_eq!(result.ineligible_evidence_ids, vec!["ev-current"]);
    assert_eq!(decision.maturity, DecisionMaturity::Verified);
}

#[test]
fn hypothesis_with_current_attestation_cannot_satisfy_current_verified_gate() {
    let (record, policy, attestation) = current_fixture(EvidenceClass::Hypothesis);
    let decision = decision(&record.id, DecisionMaturity::Experimental);
    let (evidence, attestations, policies) = maps(record, policy, attestation);

    let result = evaluate_current_verification(
        &decision,
        &evidence,
        &attestations,
        &policies,
        ts(11, 30, 0),
    )
    .unwrap();

    assert_eq!(result.status, CurrentVerificationStatus::Ineligible);
    assert_eq!(result.ineligible_evidence_ids, vec!["ev-current"]);
}

#[test]
fn renewed_attestation_restores_current_eligibility() {
    let (record, policy, expired_attestation) = current_fixture(EvidenceClass::Documented);
    let historical_decision = decision(&record.id, DecisionMaturity::Verified);
    let evidence = BTreeMap::from([(record.id.clone(), record.clone())]);
    let policies = BTreeMap::from([(policy.id.clone(), policy.clone())]);
    let expired = BTreeMap::from([(expired_attestation.evidence_id.clone(), expired_attestation)]);

    let stale = evaluate_current_verification(
        &historical_decision,
        &evidence,
        &expired,
        &policies,
        ts(12, 0, 2),
    )
    .unwrap();
    assert_eq!(stale.status, CurrentVerificationStatus::Ineligible);

    let next = evidence_record_for_renewal(&record);
    let proposal = RefreshProposal {
        previous_evidence_id: record.id.clone(),
        refreshed_evidence: next.clone(),
        source_version: "v1".into(),
        proposed_at: ts(12, 0, 3),
    };
    let prior = current_fixture(EvidenceClass::Documented).2;
    let evaluation = evaluate_lease(
        &record,
        &policy,
        RiskTier::Low,
        ts(12, 0, 3),
        Some(&prior),
        Some(&proposal),
        false,
    )
    .unwrap();
    let renewed = attest(&record, &policy, &proposal, &evaluation).unwrap();
    let renewed_decision = decision(&next.id, DecisionMaturity::Verified);
    let renewed_evidence = BTreeMap::from([(next.id.clone(), next)]);
    let renewed_attestations = BTreeMap::from([(renewed.evidence_id.clone(), renewed)]);

    let current = evaluate_current_verification(
        &renewed_decision,
        &renewed_evidence,
        &renewed_attestations,
        &policies,
        ts(12, 30, 0),
    )
    .unwrap();
    assert_eq!(current.status, CurrentVerificationStatus::Eligible);
}

fn evidence_record_for_renewal(previous: &EvidenceRecord) -> EvidenceRecord {
    evidence("ev-renewed", previous.evidence_class, "same content")
}

#[test]
fn wrong_evidence_fingerprint_is_ineligible() {
    let (record, policy, mut attestation) = current_fixture(EvidenceClass::RepoObserved);
    attestation.evidence_fingerprint = "f".repeat(64);
    let decision = decision(&record.id, DecisionMaturity::Verified);
    let (evidence, attestations, policies) = maps(record, policy, attestation);

    let result = evaluate_current_verification(
        &decision,
        &evidence,
        &attestations,
        &policies,
        ts(11, 30, 0),
    )
    .unwrap();

    assert_eq!(result.status, CurrentVerificationStatus::Ineligible);
}

#[test]
fn stale_policy_fingerprint_is_ineligible() {
    let (record, _old_policy, attestation) = current_fixture(EvidenceClass::RepoObserved);
    let current_policy = policy("2", 1800);
    let decision = decision(&record.id, DecisionMaturity::Verified);
    let (evidence, attestations, policies) = maps(record, current_policy, attestation);

    let result = evaluate_current_verification(
        &decision,
        &evidence,
        &attestations,
        &policies,
        ts(11, 30, 0),
    )
    .unwrap();

    assert_eq!(result.status, CurrentVerificationStatus::Ineligible);
}
