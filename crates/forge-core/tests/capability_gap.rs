use forge_core::{
    discover_candidates, evaluate_candidate, propose_candidate_writes, CandidateEvaluation,
    CandidateKind, GapKind, GapObservation, PromotionDecision,
};

fn gap(kind: GapKind, id: &str, summary: &str) -> GapObservation {
    GapObservation {
        id: id.to_string(),
        objective_id: "objective-polyglot".to_string(),
        kind,
        summary: summary.to_string(),
        evidence_refs: vec!["evidence-1".to_string()],
        frequency: 3,
        severity: 80,
        confidence: 90,
    }
}

#[test]
fn procedure_gaps_become_disabled_skill_candidates() {
    let candidates = discover_candidates(&[gap(
        GapKind::ReusableProcedure,
        "gap-review-loop",
        "Repeated review tasks miss the repository verification sequence",
    )])
    .expect("valid gap should produce a candidate");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].kind, CandidateKind::Skill);
    assert!(candidates[0]
        .artifacts
        .iter()
        .any(|artifact| { artifact.path == ".cline/candidates/skills/gap-review-loop/SKILL.md" }));
    assert!(candidates[0]
        .artifacts
        .iter()
        .all(|artifact| !artifact.path.starts_with(".cline/skills/")));
}

#[test]
fn external_capability_gaps_become_mcp_candidates() {
    let candidates = discover_candidates(&[gap(
        GapKind::ExternalCapability,
        "gap-browser-evidence",
        "Repository work needs a missing external browser evidence capability",
    )])
    .expect("valid gap should produce a candidate");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].kind, CandidateKind::McpServer);
    assert!(candidates[0]
        .artifacts
        .iter()
        .any(|artifact| artifact.path == ".cline/mcp/generated/gap-browser-evidence.json"));
}

#[test]
fn candidate_artifacts_are_write_proposals_not_execution_authority() {
    let candidate = discover_candidates(&[gap(
        GapKind::ReusableProcedure,
        "gap-release-check",
        "Release checks are inconsistently applied",
    )])
    .expect("valid gap")
    .remove(0);

    let actions = propose_candidate_writes(&candidate, "task-1", "gap-forge")
        .expect("candidate artifacts should become proposals");

    assert_eq!(actions.len(), candidate.artifacts.len());
    for action in actions {
        assert_eq!(action.action_type.as_str(), "write_file");
        assert_eq!(action.risk.as_str(), "medium");
        assert!(action
            .capabilities
            .iter()
            .any(|cap| cap.as_str() == "write_file"));
        assert_eq!(action.payload["operation"], "write_file");
        assert!(action.payload["path"]
            .as_str()
            .is_some_and(|path| path.starts_with(".cline/")));
        assert_eq!(action.payload["approved"], serde_json::Value::Null);
    }
}

#[test]
fn promotion_requires_improvement_and_zero_safety_regressions() {
    let promote = evaluate_candidate(&CandidateEvaluation {
        candidate_id: "gap-review-loop".to_string(),
        baseline_success_bps: 7000,
        candidate_success_bps: 7600,
        safety_regressions: 0,
        evidence_refs: vec!["eval-1".to_string()],
    });
    assert_eq!(promote, PromotionDecision::Promote);

    let no_improvement = evaluate_candidate(&CandidateEvaluation {
        candidate_id: "gap-review-loop".to_string(),
        baseline_success_bps: 7600,
        candidate_success_bps: 7600,
        safety_regressions: 0,
        evidence_refs: vec!["eval-2".to_string()],
    });
    assert_eq!(no_improvement, PromotionDecision::RejectNoImprovement);

    let unsafe_candidate = evaluate_candidate(&CandidateEvaluation {
        candidate_id: "gap-review-loop".to_string(),
        baseline_success_bps: 7000,
        candidate_success_bps: 9000,
        safety_regressions: 1,
        evidence_refs: vec!["eval-3".to_string()],
    });
    assert_eq!(unsafe_candidate, PromotionDecision::RejectSafetyRegression);
}

#[test]
fn unsafe_candidate_ids_are_rejected_before_paths_are_generated() {
    let result = discover_candidates(&[gap(
        GapKind::ReusableProcedure,
        "../escape",
        "Attempt to escape the candidate namespace",
    )]);

    assert!(result.is_err());
}
