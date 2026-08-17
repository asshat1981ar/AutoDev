use forge_core::{
    EvalTask, EvaluationError, ProtectedSurface, TaskSource, TaskSourceKind, VerificationRecipe,
    VerifierStep,
};

fn task() -> EvalTask {
    EvalTask {
        id: "sample-task".into(),
        source: TaskSource {
            kind: TaskSourceKind::MergedPullRequest,
            repository: "asshat1981ar/AutoDev".into(),
            source_ref: "6df35bf674af8023779f59b6770135dca2895d74".into(),
            source_url: Some("https://github.com/asshat1981ar/AutoDev/pull/9".into()),
        },
        base_sha: "5c0adf94d192aef131c96d4cb72ef00e30bf7501".into(),
        specification: "Implement normalized architecture evidence contracts".into(),
        acceptance_criteria: vec!["focused verifier passes".into()],
        verifier: VerificationRecipe {
            steps: vec![VerifierStep {
                id: "rust-test".into(),
                program: "cargo".into(),
                args: vec!["test".into(), "-p".into(), "forge-core".into()],
                working_directory: "crates".into(),
                timeout_seconds: 600,
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

#[test]
fn valid_task_validates_and_has_stable_key() {
    let first = task();
    let second = task();
    first.validate().unwrap();
    second.validate().unwrap();
    assert_eq!(first.key().unwrap(), second.key().unwrap());
}

#[test]
fn base_sha_must_be_full_hex_sha() {
    let mut value = task();
    value.base_sha = "main".into();
    assert!(matches!(
        value.validate(),
        Err(EvaluationError::InvalidGitSha {
            field: "base_sha",
            ..
        })
    ));
}

#[test]
fn source_ref_must_be_full_hex_sha() {
    let mut value = task();
    value.source.source_ref = "01234567890123456789012345678901234567zz".into();
    assert!(matches!(
        value.validate(),
        Err(EvaluationError::InvalidGitSha {
            field: "source_ref",
            ..
        })
    ));
}

#[test]
fn empty_verifier_is_rejected() {
    let mut value = task();
    value.verifier.steps.clear();
    assert_eq!(
        value.validate().unwrap_err(),
        EvaluationError::EmptyVerifier("sample-task".into())
    );
}

#[test]
fn zero_timeout_is_rejected() {
    let mut value = task();
    value.verifier.steps[0].timeout_seconds = 0;
    assert_eq!(
        value.validate().unwrap_err(),
        EvaluationError::InvalidTimeout("rust-test".into())
    );
}

#[test]
fn opaque_shell_wrappers_are_rejected() {
    for (program, args) in [
        ("bash", vec!["-c", "cargo test"]),
        ("sh", vec!["-c", "cargo test"]),
        ("powershell", vec!["-Command", "cargo test"]),
        ("cmd.exe", vec!["/c", "cargo test"]),
    ] {
        let mut value = task();
        value.verifier.steps[0].program = program.into();
        value.verifier.steps[0].args = args.into_iter().map(String::from).collect();
        assert!(matches!(
            value.validate(),
            Err(EvaluationError::OpaqueShell { .. })
        ));
    }
}

#[test]
fn unsafe_verifier_working_directories_are_rejected() {
    for working_directory in ["../outside", "/tmp/outside"] {
        let mut value = task();
        value.verifier.steps[0].working_directory = working_directory.into();
        assert!(matches!(
            value.validate(),
            Err(EvaluationError::UnsafePath {
                field: "working_directory",
                ..
            })
        ));
    }
}

#[test]
fn protected_surface_cannot_overlap_expected_change_scope() {
    let mut value = task();
    value.protected.paths = vec!["crates/forge-core/tests/".into()];
    assert!(matches!(
        value.validate(),
        Err(EvaluationError::ProtectedScopeOverlap { .. })
    ));
}

#[test]
fn verifier_asset_fingerprint_must_be_full_sha256() {
    let mut value = task();
    value.verifier.asset_fingerprints = vec!["abc".into()];
    assert_eq!(
        value.validate().unwrap_err(),
        EvaluationError::InvalidVerifierAssetFingerprint("abc".into())
    );
}

#[test]
fn changing_asset_fingerprint_changes_verifier_fingerprint() {
    let mut first = task();
    first.verifier.asset_fingerprints = vec!["a".repeat(64)];
    let mut second = task();
    second.verifier.asset_fingerprints = vec!["b".repeat(64)];
    assert_ne!(
        first.verifier_fingerprint().unwrap(),
        second.verifier_fingerprint().unwrap()
    );
}

#[test]
fn set_like_fields_are_canonicalized_for_task_fingerprint() {
    let mut first = task();
    first.acceptance_criteria = vec!["b".into(), "a".into(), "a".into()];
    first.protected.paths = vec!["z/".into(), ".autodev-eval/".into()];
    first.expected_change_scope = vec!["crates/forge-core/".into(), "docs/".into()];
    first.verifier.asset_fingerprints = vec!["b".repeat(64), "a".repeat(64)];

    let mut second = task();
    second.acceptance_criteria = vec!["a".into(), "b".into()];
    second.protected.paths = vec![".autodev-eval/".into(), "z/".into()];
    second.expected_change_scope = vec!["docs/".into(), "crates/forge-core/".into()];
    second.verifier.asset_fingerprints = vec!["a".repeat(64), "b".repeat(64)];

    assert_eq!(
        first.task_fingerprint().unwrap(),
        second.task_fingerprint().unwrap()
    );
}

#[test]
fn verifier_step_order_is_fingerprint_significant() {
    let second_step = VerifierStep {
        id: "format".into(),
        program: "cargo".into(),
        args: vec!["fmt".into(), "--all".into(), "--".into(), "--check".into()],
        working_directory: "crates".into(),
        timeout_seconds: 120,
        required: true,
    };
    let mut first = task();
    first.verifier.steps.push(second_step.clone());
    let mut second = task();
    second.verifier.steps.insert(0, second_step);

    assert_ne!(
        first.verifier_fingerprint().unwrap(),
        second.verifier_fingerprint().unwrap()
    );
}
