use chrono::{TimeZone, Utc};
use forge_core::{
    render_architecture_report, ArchitectureAlternative, ArchitectureDecision,
    ArchitectureEvidenceError, ArchitectureReportInput, DecisionMaturity, EvidenceClass,
    EvidenceRecord, Reversibility,
};

#[test]
fn architecture_evidence_is_gate_aware_and_objective_scoped() {
    let evidence = EvidenceRecord::new(
        "ev-1",
        "obj-eval",
        "ForgeCore evidence is normalized",
        EvidenceClass::RepoObserved,
        "fixture",
        "fixture://architecture-evidence",
        Utc.with_ymd_and_hms(2026, 8, 17, 8, 0, 0).unwrap(),
        90,
        "normalized architecture evidence",
        "repository semantics change",
    )
    .unwrap();
    assert!(evidence.can_satisfy_verified_gate());

    let decision = ArchitectureDecision {
        id: "dec-cross-objective".into(),
        objective_id: "obj-other".into(),
        decision: "Keep evidence scoped to an objective".into(),
        alternatives: vec![ArchitectureAlternative {
            name: "global evidence".into(),
            rationale: "would blur objective boundaries".into(),
            rejected: true,
        }],
        contradiction: "reuse vs isolation".into(),
        selected_option: "objective-scoped evidence".into(),
        rationale: "prevents cross-objective evidence leakage".into(),
        evidence_refs: vec!["ev-1".into()],
        reversibility: Reversibility::Easy,
        risks: vec![],
        invalidation_conditions: vec!["objective isolation is removed by design".into()],
        maturity: DecisionMaturity::Verified,
    };

    let input = ArchitectureReportInput {
        objective_id: "obj-eval".into(),
        title: "Evaluation probe".into(),
        desired_outcome: "Evidence stays objective-scoped".into(),
        evidence: vec![evidence],
        decisions: vec![decision],
        options: vec![],
    };

    assert!(matches!(
        render_architecture_report(&input),
        Err(ArchitectureEvidenceError::ObjectiveMismatch {
            item_kind: "decision",
            ..
        })
    ));
}
