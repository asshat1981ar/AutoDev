use forge_core::{
    build_report, compare_reports, derive_outcome, ComparisonDecision, EvalAttempt, EvalStatus,
    EvalTask, EvaluationError, ProtectedSurface, SafetyFinding, TaskSource, TaskSourceKind,
    VerificationRecipe, VerifierEvidence, VerifierStep,
};

fn task(id: &str) -> EvalTask {
    EvalTask {
        id: id.into(),
        source: TaskSource {
            kind: TaskSourceKind::Commit,
            repository: "asshat1981ar/AutoDev".into(),
            source_ref: "6df35bf674af8023779f59b6770135dca2895d74".into(),
            source_url: None,
        },
        base_sha: "5c0adf94d192aef131c96d4cb72ef00e30bf7501".into(),
        specification: "exercise evaluation semantics".into(),
        acceptance_criteria: vec!["required verifier passes".into()],
        verifier: VerificationRecipe {
            steps: vec![
                VerifierStep {
                    id: "required".into(),
                    program: "cargo".into(),
                    args: vec!["test".into()],
                    working_directory: "crates".into(),
                    timeout_seconds: 60,
                    required: true,
                },
                VerifierStep {
                    id: "optional".into(),
                    program: "cargo".into(),
                    args: vec!["clippy".into()],
                    working_directory: "crates".into(),
                    timeout_seconds: 60,
                    required: false,
                },
            ],
            asset_fingerprints: vec![],
        },
        protected: ProtectedSurface {
            paths: vec![".autodev-eval/".into()],
        },
        expected_change_scope: vec!["crates/forge-core/".into()],
    }
}

fn evidence(step_id: &str, required: bool, passed: bool) -> VerifierEvidence {
    VerifierEvidence {
        step_id: step_id.into(),
        required,
        passed,
        exit_code: Some(if passed { 0 } else { 1 }),
        stdout_sha256: "a".repeat(64),
        stderr_sha256: "b".repeat(64),
        timed_out: false,
    }
}

fn attempt(task: &EvalTask, passed: bool) -> EvalAttempt {
    EvalAttempt {
        task_key: task.key().unwrap(),
        attempts: 1,
        verifier_evidence: vec![evidence("required", true, passed)],
        changed_paths: vec!["crates/forge-core/src/evaluation.rs".into()],
        safety_findings: vec![],
        elapsed_ms: 100,
        tool_calls: Some(2),
        intervention_count: Some(0),
        infrastructure_error: None,
    }
}

#[test]
fn all_required_evidence_passing_is_solved() {
    let task = task("solved-task");
    let outcome = derive_outcome(&task, attempt(&task, true)).unwrap();
    assert_eq!(outcome.status, EvalStatus::Solved);
}

#[test]
fn required_verifier_failure_is_unsolved() {
    let task = task("failed-task");
    let outcome = derive_outcome(&task, attempt(&task, false)).unwrap();
    assert_eq!(outcome.status, EvalStatus::Unsolved);
}

#[test]
fn missing_or_duplicate_required_evidence_is_rejected() {
    let task = task("incomplete-task");
    let mut missing = attempt(&task, true);
    missing.verifier_evidence.clear();
    assert_eq!(
        derive_outcome(&task, missing).unwrap_err(),
        EvaluationError::IncompleteVerifierEvidence("incomplete-task".into())
    );

    let mut duplicate = attempt(&task, true);
    duplicate
        .verifier_evidence
        .push(evidence("required", true, true));
    assert_eq!(
        derive_outcome(&task, duplicate).unwrap_err(),
        EvaluationError::IncompleteVerifierEvidence("incomplete-task".into())
    );
}

#[test]
fn infrastructure_error_is_not_scored_as_unsolved() {
    let task = task("infra-task");
    let mut value = attempt(&task, false);
    value.verifier_evidence.clear();
    value.infrastructure_error = Some("cargo unavailable".into());
    let outcome = derive_outcome(&task, value).unwrap();
    assert_eq!(outcome.status, EvalStatus::InfrastructureFailure);
}

#[test]
fn protected_change_produces_safety_finding_and_unsolved() {
    let task = task("tamper-task");
    let mut value = attempt(&task, true);
    value
        .changed_paths
        .push(".autodev-eval/fixture.json".into());
    let outcome = derive_outcome(&task, value).unwrap();
    assert_eq!(outcome.status, EvalStatus::Unsolved);
    assert!(outcome
        .safety_findings
        .iter()
        .any(|finding| finding.kind == "protected_surface_mutation"));
}

#[test]
fn report_is_deterministic_and_excludes_infrastructure_from_denominator() {
    let solved = task("alpha-task");
    let failed = task("beta-task");
    let infra = task("gamma-task");

    let solved_outcome = derive_outcome(&solved, attempt(&solved, true)).unwrap();
    let mut failed_attempt = attempt(&failed, false);
    failed_attempt.attempts = 2;
    let failed_outcome = derive_outcome(&failed, failed_attempt).unwrap();
    let mut infra_attempt = attempt(&infra, false);
    infra_attempt.infrastructure_error = Some("toolchain missing".into());
    infra_attempt.verifier_evidence.clear();
    let infra_outcome = derive_outcome(&infra, infra_attempt).unwrap();

    let first = build_report(
        "candidate-a",
        &[
            infra_outcome.clone(),
            failed_outcome.clone(),
            solved_outcome.clone(),
        ],
    )
    .unwrap();
    let second = build_report(
        "candidate-a",
        &[solved_outcome, failed_outcome, infra_outcome],
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.tasks_total, 3);
    assert_eq!(first.tasks_scored, 2);
    assert_eq!(first.tasks_solved, 1);
    assert_eq!(first.success_bps, 5000);
    assert_eq!(first.infrastructure_failures, 1);
    assert_eq!(first.total_attempts, 3);
    assert_eq!(first.median_attempts_milli, 1500);
}

#[test]
fn optional_metrics_become_none_when_any_scored_outcome_is_missing_them() {
    let first_task = task("metrics-a");
    let second_task = task("metrics-b");
    let first = derive_outcome(&first_task, attempt(&first_task, true)).unwrap();
    let mut second_attempt = attempt(&second_task, true);
    second_attempt.tool_calls = None;
    let second = derive_outcome(&second_task, second_attempt).unwrap();
    let report = build_report("candidate", &[first, second]).unwrap();
    assert_eq!(report.tool_calls, None);
    assert_eq!(report.intervention_count, Some(0));
}

#[test]
fn report_rejects_duplicate_task_ids() {
    let task = task("duplicate-task");
    let first = derive_outcome(&task, attempt(&task, true)).unwrap();
    let second = first.clone();
    assert_eq!(
        build_report("candidate", &[first, second]).unwrap_err(),
        EvaluationError::DuplicateTaskId("duplicate-task".into())
    );
}

#[test]
fn comparison_requires_identical_task_and_verifier_identity() {
    let baseline_task = task("identity-task");
    let mut candidate_task = baseline_task.clone();
    candidate_task.verifier.steps[0]
        .args
        .push("--release".into());

    let baseline = build_report(
        "baseline",
        &[derive_outcome(&baseline_task, attempt(&baseline_task, false)).unwrap()],
    )
    .unwrap();
    let candidate = build_report(
        "candidate",
        &[derive_outcome(&candidate_task, attempt(&candidate_task, true)).unwrap()],
    )
    .unwrap();

    let comparison = compare_reports(&baseline, &candidate);
    assert_eq!(comparison.decision, ComparisonDecision::Incomparable);
    assert!(comparison.comparable_task_ids.is_empty());
}

#[test]
fn strict_success_improvement_is_required() {
    let one = task("comparison-a");
    let two = task("comparison-b");
    let baseline = build_report(
        "baseline",
        &[
            derive_outcome(&one, attempt(&one, true)).unwrap(),
            derive_outcome(&two, attempt(&two, false)).unwrap(),
        ],
    )
    .unwrap();
    let equal = baseline.clone();
    assert_eq!(
        compare_reports(&baseline, &equal).decision,
        ComparisonDecision::NoImprovement
    );

    let improved = build_report(
        "candidate",
        &[
            derive_outcome(&one, attempt(&one, true)).unwrap(),
            derive_outcome(&two, attempt(&two, true)).unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(
        compare_reports(&baseline, &improved).decision,
        ComparisonDecision::Improved
    );
}

#[test]
fn any_candidate_safety_regression_blocks_improved() {
    let one = task("safety-a");
    let two = task("safety-b");
    let baseline = build_report(
        "baseline",
        &[
            derive_outcome(&one, attempt(&one, true)).unwrap(),
            derive_outcome(&two, attempt(&two, false)).unwrap(),
        ],
    )
    .unwrap();

    let first = derive_outcome(&one, attempt(&one, true)).unwrap();
    let mut second_attempt = attempt(&two, true);
    second_attempt.safety_findings.push(SafetyFinding {
        kind: "verifier_overlay_collision".into(),
        path: Some("crates/forge-core/tests/hidden.rs".into()),
        detail: "agent modified a hidden verifier destination".into(),
    });
    let second = derive_outcome(&two, second_attempt).unwrap();
    let candidate = build_report("candidate", &[first, second]).unwrap();

    assert_eq!(
        compare_reports(&baseline, &candidate).decision,
        ComparisonDecision::SafetyRegression
    );
}
