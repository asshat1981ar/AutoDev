use serde::{Deserialize, Serialize};

use super::HarnessProfile;

/// Externally measured baseline-versus-candidate harness metrics.
///
/// Rates are integer basis points (`10_000 == 100%`). Duration and resource
/// units are intentionally generic integer counters so the evaluator remains
/// deterministic, portable, and free of floating-point threshold behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessEvaluation {
    pub sample_size: u32,
    pub baseline_correctness_bps: u16,
    pub candidate_correctness_bps: u16,
    pub baseline_evidence_completion_bps: u16,
    pub candidate_evidence_completion_bps: u16,
    pub baseline_unsafe_action_rejection_bps: u16,
    pub candidate_unsafe_action_rejection_bps: u16,
    pub baseline_duration_ms: u64,
    pub candidate_duration_ms: u64,
    pub baseline_resource_units: u64,
    pub candidate_resource_units: u64,
    #[serde(default)]
    pub independent_verification_refs: Vec<String>,
}

/// Advisory result for a harness candidate evaluation.
///
/// This decision carries no execution, authorization, registry-mutation, or
/// policy-mutation authority. Promotion still requires the repository's normal
/// review and ForgeCore-controlled integration path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessPromotionDecision {
    Eligible,
    RejectInvalidProfile,
    RejectInvalidMetrics,
    RejectZeroSamples,
    RejectIndependentVerification,
    RejectCorrectnessRegression,
    RejectEvidenceCompletionRegression,
    RejectUnsafeActionRejectionRegression,
    RejectNoEfficiencyImprovement,
}

/// Evaluate whether a harness candidate has earned an advisory promotion signal.
///
/// Safety, correctness, and measured efficiency dimensions are non-regression
/// gates. Independent verification and a non-zero sample are mandatory. At
/// least one efficiency metric must improve strictly while the other does not
/// regress. The function is pure: it does not mutate the supplied profile,
/// registries, policy state, capability grants, or execution state.
pub fn evaluate_harness_candidate(
    profile: &HarnessProfile,
    evaluation: &HarnessEvaluation,
) -> HarnessPromotionDecision {
    if profile.validate().is_err() {
        return HarnessPromotionDecision::RejectInvalidProfile;
    }

    if !rates_are_valid(evaluation) {
        return HarnessPromotionDecision::RejectInvalidMetrics;
    }

    if evaluation.sample_size == 0 {
        return HarnessPromotionDecision::RejectZeroSamples;
    }

    if evaluation.independent_verification_refs.is_empty()
        || evaluation
            .independent_verification_refs
            .iter()
            .any(|reference| reference.trim().is_empty())
    {
        return HarnessPromotionDecision::RejectIndependentVerification;
    }

    if evaluation.candidate_correctness_bps < evaluation.baseline_correctness_bps {
        return HarnessPromotionDecision::RejectCorrectnessRegression;
    }

    if evaluation.candidate_evidence_completion_bps < evaluation.baseline_evidence_completion_bps {
        return HarnessPromotionDecision::RejectEvidenceCompletionRegression;
    }

    if evaluation.candidate_unsafe_action_rejection_bps
        < evaluation.baseline_unsafe_action_rejection_bps
    {
        return HarnessPromotionDecision::RejectUnsafeActionRejectionRegression;
    }

    let duration_improved = evaluation.candidate_duration_ms < evaluation.baseline_duration_ms;
    let resources_improved =
        evaluation.candidate_resource_units < evaluation.baseline_resource_units;
    let duration_regressed = evaluation.candidate_duration_ms > evaluation.baseline_duration_ms;
    let resources_regressed =
        evaluation.candidate_resource_units > evaluation.baseline_resource_units;
    if (!duration_improved && !resources_improved) || duration_regressed || resources_regressed {
        return HarnessPromotionDecision::RejectNoEfficiencyImprovement;
    }

    HarnessPromotionDecision::Eligible
}

fn rates_are_valid(evaluation: &HarnessEvaluation) -> bool {
    [
        evaluation.baseline_correctness_bps,
        evaluation.candidate_correctness_bps,
        evaluation.baseline_evidence_completion_bps,
        evaluation.candidate_evidence_completion_bps,
        evaluation.baseline_unsafe_action_rejection_bps,
        evaluation.candidate_unsafe_action_rejection_bps,
    ]
    .into_iter()
    .all(|rate| rate <= 10_000)
}
