use chrono::{TimeZone, Utc};
use forge_core::{
    render_architecture_report, ArchitectureDecision, ArchitectureEvidenceError,
    ArchitectureReportInput, DecisionMaturity, EvidenceClass, EvidenceRecord, Reversibility,
};

fn evidence(id: &str, objective_id: &str, class: EvidenceClass) -> EvidenceRecord {
    EvidenceRecord::new(
        id,
        objective_id,
        "Normalized architecture claim",
        class,
        "fixture",
        "fixture://source",
        Utc.with_ymd_and_hms(2026, 8, 17, 8, 0, 0).unwrap(),
        90,
        "normalized content",
        "source changes materially",
    )
    .unwrap()
}

#[test]
fn public_record_with_empty_id_is_rejected() {
    let mut record = evidence("ev-1", "obj-w1", EvidenceClass::Documented);
    record.id.clear();

    assert_eq!(
        record.validate().unwrap_err(),
        ArchitectureEvidenceError::EmptyField("id"),
    );
}

#[test]
fn public_record_with_malformed_fingerprint_is_rejected() {
    let mut record = evidence("ev-1", "obj-w1", EvidenceClass::Documented);
    record.content_fingerprint = "not-a-sha256".into();

    assert_eq!(
        record.validate().unwrap_err(),
        ArchitectureEvidenceError::InvalidFingerprint("not-a-sha256".into()),
    );
}

#[test]
fn renderer_rejects_malformed_public_record() {
    let mut malformed = evidence("ev-1", "obj-w1", EvidenceClass::Documented);
    malformed.source_system.clear();
    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "Malformed evidence".into(),
        desired_outcome: "Deserialized records cannot bypass invariants".into(),
        evidence: vec![malformed],
        decisions: vec![],
        options: vec![],
    };

    assert_eq!(
        render_architecture_report(&input).unwrap_err(),
        ArchitectureEvidenceError::EmptyField("source_system"),
    );
}

#[test]
fn report_rejects_duplicate_evidence_ids() {
    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "Duplicate evidence".into(),
        desired_outcome: "References stay unambiguous".into(),
        evidence: vec![
            evidence("ev-duplicate", "obj-w1", EvidenceClass::Documented),
            evidence("ev-duplicate", "obj-w1", EvidenceClass::ResearchSupported),
        ],
        decisions: vec![],
        options: vec![],
    };

    assert_eq!(
        render_architecture_report(&input).unwrap_err(),
        ArchitectureEvidenceError::DuplicateEvidenceId("ev-duplicate".into()),
    );
}

#[test]
fn report_rejects_evidence_from_another_objective() {
    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "Objective isolation".into(),
        desired_outcome: "Evidence cannot cross objective boundaries".into(),
        evidence: vec![evidence("ev-other", "obj-other", EvidenceClass::Documented)],
        decisions: vec![],
        options: vec![],
    };

    assert_eq!(
        render_architecture_report(&input).unwrap_err(),
        ArchitectureEvidenceError::ObjectiveMismatch {
            item_kind: "evidence",
            item_id: "ev-other".into(),
            expected: "obj-w1".into(),
            actual: "obj-other".into(),
        },
    );
}

#[test]
fn report_rejects_decision_from_another_objective() {
    let supported = evidence("ev-1", "obj-w1", EvidenceClass::RepoObserved);
    let decision = ArchitectureDecision {
        id: "dec-other".into(),
        objective_id: "obj-other".into(),
        decision: "Use another objective's decision".into(),
        alternatives: vec![forge_core::ArchitectureAlternative {
            name: "local decision".into(),
            rationale: "keeps scope isolated".into(),
            rejected: true,
        }],
        contradiction: "reuse vs scope isolation".into(),
        selected_option: "foreign decision".into(),
        rationale: "invalid cross-objective fixture".into(),
        evidence_refs: vec!["ev-1".into()],
        reversibility: Reversibility::Easy,
        risks: vec![],
        invalidation_conditions: vec!["objective IDs become global by design".into()],
        maturity: DecisionMaturity::Verified,
    };
    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "Decision isolation".into(),
        desired_outcome: "Decisions stay scoped to their objective".into(),
        evidence: vec![supported],
        decisions: vec![decision],
        options: vec![],
    };

    assert_eq!(
        render_architecture_report(&input).unwrap_err(),
        ArchitectureEvidenceError::ObjectiveMismatch {
            item_kind: "decision",
            item_id: "dec-other".into(),
            expected: "obj-w1".into(),
            actual: "obj-other".into(),
        },
    );
}
