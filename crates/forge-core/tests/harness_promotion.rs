use forge_core::{
    default_harness_profiles, evaluate_harness_candidate, HarnessEvaluation,
    HarnessPromotionDecision,
};

fn profile() -> forge_core::HarnessProfile {
    default_harness_profiles()
        .get("forgeflow-sdlc")
        .expect("built-in profile")
        .clone()
}

fn evaluation() -> HarnessEvaluation {
    HarnessEvaluation {
        sample_size: 100,
        baseline_correctness_bps: 9_500,
        candidate_correctness_bps: 9_600,
        baseline_evidence_completion_bps: 9_000,
        candidate_evidence_completion_bps: 9_100,
        baseline_unsafe_action_rejection_bps: 10_000,
        candidate_unsafe_action_rejection_bps: 10_000,
        baseline_duration_ms: 1_000,
        candidate_duration_ms: 900,
        baseline_resource_units: 100,
        candidate_resource_units: 100,
        independent_verification_refs: vec!["ci:run-595".to_string()],
    }
}

#[test]
fn promotion_is_eligible_only_with_non_regression_and_efficiency_gain() {
    assert_eq!(
        evaluate_harness_candidate(&profile(), &evaluation()),
        HarnessPromotionDecision::Eligible
    );
}

#[test]
fn promotion_rejects_correctness_regression() {
    let mut candidate = evaluation();
    candidate.candidate_correctness_bps = candidate.baseline_correctness_bps - 1;

    assert_eq!(
        evaluate_harness_candidate(&profile(), &candidate),
        HarnessPromotionDecision::RejectCorrectnessRegression
    );
}

#[test]
fn promotion_rejects_evidence_completion_regression() {
    let mut candidate = evaluation();
    candidate.candidate_evidence_completion_bps = candidate.baseline_evidence_completion_bps - 1;

    assert_eq!(
        evaluate_harness_candidate(&profile(), &candidate),
        HarnessPromotionDecision::RejectEvidenceCompletionRegression
    );
}

#[test]
fn promotion_rejects_unsafe_action_rejection_regression() {
    let mut candidate = evaluation();
    candidate.candidate_unsafe_action_rejection_bps =
        candidate.baseline_unsafe_action_rejection_bps - 1;

    assert_eq!(
        evaluate_harness_candidate(&profile(), &candidate),
        HarnessPromotionDecision::RejectUnsafeActionRejectionRegression
    );
}

#[test]
fn promotion_rejects_zero_samples() {
    let mut candidate = evaluation();
    candidate.sample_size = 0;

    assert_eq!(
        evaluate_harness_candidate(&profile(), &candidate),
        HarnessPromotionDecision::RejectZeroSamples
    );
}

#[test]
fn promotion_rejects_self_reported_only_verification() {
    let mut candidate = evaluation();
    candidate.independent_verification_refs.clear();

    assert_eq!(
        evaluate_harness_candidate(&profile(), &candidate),
        HarnessPromotionDecision::RejectIndependentVerification
    );
}

#[test]
fn promotion_requires_at_least_one_efficiency_improvement() {
    let mut candidate = evaluation();
    candidate.candidate_duration_ms = candidate.baseline_duration_ms;
    candidate.candidate_resource_units = candidate.baseline_resource_units;

    assert_eq!(
        evaluate_harness_candidate(&profile(), &candidate),
        HarnessPromotionDecision::RejectNoEfficiencyImprovement
    );
}
