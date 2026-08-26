use forge_core::{
    build_report, derive_outcome, EvalAttempt, EvalStatus, EvalTask, EvaluationError,
    ProtectedSurface, TaskSource, TaskSourceKind, VerificationRecipe, VerifierStep,
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
        specification: "exercise adversarial evaluation invariants".into(),
        acceptance_criteria: vec!["required verifier passes".into()],
        verifier: VerificationRecipe {
            steps: vec![VerifierStep {
                id: "required".into(),
                program: "cargo".into(),
                args: vec!["test".into()],
                working_directory: "crates".into(),
                timeout_seconds: 60,
                required: true,
            }],
            asset_fingerprints: vec![],
        },
        protected: ProtectedSurface {
            paths: vec![".autodev-eval/".into()],
        },
        expected_change_scope: vec!["crates/forge-core/".into()],
    }
}

fn attempt(task: &EvalTask) -> EvalAttempt {
    EvalAttempt {
        task_key: task.key().unwrap(),
        attempts: 1,
        verifier_evidence: vec![],
        changed_paths: vec![],
        safety_findings: vec![],
        elapsed_ms: 1,
        tool_calls: None,
        intervention_count: None,
        infrastructure_error: None,
    }
}

#[test]
fn caller_cannot_claim_solved_without_required_verifier_evidence() {
    let task = task("no-evidence");
    assert_eq!(
        derive_outcome(&task, attempt(&task)).unwrap_err(),
        EvaluationError::IncompleteVerifierEvidence("no-evidence".into())
    );
}

#[test]
fn all_infrastructure_failures_produce_zero_scored_denominator() {
    let first = task("infra-a");
    let second = task("infra-b");

    let mut first_attempt = attempt(&first);
    first_attempt.infrastructure_error = Some("git unavailable".into());
    let mut second_attempt = attempt(&second);
    second_attempt.infrastructure_error = Some("toolchain unavailable".into());

    let first_outcome = derive_outcome(&first, first_attempt).unwrap();
    let second_outcome = derive_outcome(&second, second_attempt).unwrap();
    assert_eq!(first_outcome.status, EvalStatus::InfrastructureFailure);
    assert_eq!(second_outcome.status, EvalStatus::InfrastructureFailure);

    let report = build_report("all-infra", &[first_outcome, second_outcome]).unwrap();
    assert_eq!(report.tasks_total, 2);
    assert_eq!(report.tasks_scored, 0);
    assert_eq!(report.tasks_solved, 0);
    assert_eq!(report.success_bps, 0);
    assert_eq!(report.infrastructure_failures, 2);
    assert_eq!(report.total_attempts, 0);
}
