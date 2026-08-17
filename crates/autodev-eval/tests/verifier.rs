use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use autodev_eval::{
    apply_verifier_overlays, run_verifier, RunnerError, VerifierOverlay,
};
use forge_core::{VerificationRecipe, VerifierStep};
use sha2::{Digest, Sha256};

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn overlay(source: &str, destination: &str, digest: String) -> VerifierOverlay {
    VerifierOverlay {
        source_path: source.into(),
        destination_path: destination.into(),
        sha256: digest,
    }
}

fn executable_step(id: &str, args: Vec<String>, timeout_seconds: u32) -> VerifierStep {
    VerifierStep {
        id: id.into(),
        program: std::env::current_exe()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        args,
        working_directory: ".".into(),
        timeout_seconds,
        required: true,
    }
}

fn recipe(step: VerifierStep) -> VerificationRecipe {
    VerificationRecipe {
        steps: vec![step],
        asset_fingerprints: vec![],
    }
}

#[test]
fn wrong_overlay_digest_is_rejected_before_destination_write() {
    let crate_root = tempfile::tempdir().unwrap();
    let workspace = tempfile::tempdir().unwrap();
    fs::create_dir_all(crate_root.path().join("fixture-assets")).unwrap();
    fs::write(crate_root.path().join("fixture-assets/probe.rs"), b"probe").unwrap();

    let overlays = vec![overlay(
        "fixture-assets/probe.rs",
        "crates/forge-core/tests/probe.rs",
        "a".repeat(64),
    )];
    assert!(matches!(
        apply_verifier_overlays(crate_root.path(), workspace.path(), &overlays),
        Err(RunnerError::OverlayIntegrity(_))
    ));
    assert!(!workspace
        .path()
        .join("crates/forge-core/tests/probe.rs")
        .exists());
}

#[test]
fn valid_overlay_copies_exact_hash_verified_bytes() {
    let crate_root = tempfile::tempdir().unwrap();
    let workspace = tempfile::tempdir().unwrap();
    let bytes = b"independent verifier probe\n";
    fs::create_dir_all(crate_root.path().join("fixture-assets")).unwrap();
    fs::write(crate_root.path().join("fixture-assets/probe.rs"), bytes).unwrap();

    let overlays = vec![overlay(
        "fixture-assets/probe.rs",
        "crates/forge-core/tests/probe.rs",
        sha256(bytes),
    )];
    apply_verifier_overlays(crate_root.path(), workspace.path(), &overlays).unwrap();
    assert_eq!(
        fs::read(workspace.path().join("crates/forge-core/tests/probe.rs")).unwrap(),
        bytes
    );
}

#[test]
fn absolute_and_traversal_overlay_destinations_are_rejected() {
    let crate_root = tempfile::tempdir().unwrap();
    let workspace = tempfile::tempdir().unwrap();
    let bytes = b"probe";
    fs::create_dir_all(crate_root.path().join("fixture-assets")).unwrap();
    fs::write(crate_root.path().join("fixture-assets/probe.rs"), bytes).unwrap();

    for destination in ["../outside.rs", "/tmp/outside.rs"] {
        let overlays = vec![overlay(
            "fixture-assets/probe.rs",
            destination,
            sha256(bytes),
        )];
        assert!(matches!(
            apply_verifier_overlays(crate_root.path(), workspace.path(), &overlays),
            Err(RunnerError::UnsafeOverlayDestination(_))
        ));
    }
}

#[test]
#[cfg(unix)]
fn symlink_escape_overlay_destination_is_rejected() {
    let crate_root = tempfile::tempdir().unwrap();
    let workspace = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let bytes = b"probe";
    fs::create_dir_all(crate_root.path().join("fixture-assets")).unwrap();
    fs::write(crate_root.path().join("fixture-assets/probe.rs"), bytes).unwrap();
    std::os::unix::fs::symlink(outside.path(), workspace.path().join("escape")).unwrap();

    let overlays = vec![overlay(
        "fixture-assets/probe.rs",
        "escape/probe.rs",
        sha256(bytes),
    )];
    assert!(matches!(
        apply_verifier_overlays(crate_root.path(), workspace.path(), &overlays),
        Err(RunnerError::UnsafeOverlayDestination(_))
    ));
    assert!(!outside.path().join("probe.rs").exists());
}

#[test]
fn passing_executable_produces_execution_backed_evidence() {
    let workspace = tempfile::tempdir().unwrap();
    let executions = run_verifier(
        workspace.path(),
        &recipe(executable_step("pass", vec!["--list".into()], 10)),
    )
    .unwrap();

    assert_eq!(executions.len(), 1);
    let evidence = &executions[0].evidence;
    assert!(evidence.passed);
    assert_eq!(evidence.exit_code, Some(0));
    assert!(!evidence.timed_out);
    assert_eq!(evidence.stdout_sha256.len(), 64);
    assert_eq!(evidence.stderr_sha256.len(), 64);
    assert!(evidence.stdout_sha256.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert!(evidence.stderr_sha256.bytes().all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn nonzero_exit_is_failed_evidence_not_infrastructure_failure() {
    let workspace = tempfile::tempdir().unwrap();
    let executions = run_verifier(
        workspace.path(),
        &recipe(executable_step(
            "fail",
            vec!["--definitely-invalid-libtest-flag".into()],
            10,
        )),
    )
    .unwrap();

    let evidence = &executions[0].evidence;
    assert!(!evidence.passed);
    assert_ne!(evidence.exit_code, Some(0));
    assert!(!evidence.timed_out);
}

#[test]
fn missing_verifier_executable_is_an_infrastructure_error() {
    let workspace = tempfile::tempdir().unwrap();
    let recipe = recipe(VerifierStep {
        id: "missing".into(),
        program: "autodev-eval-definitely-missing-executable".into(),
        args: vec![],
        working_directory: ".".into(),
        timeout_seconds: 10,
        required: true,
    });

    assert!(matches!(
        run_verifier(workspace.path(), &recipe),
        Err(RunnerError::MissingExecutable(name))
            if name == "autodev-eval-definitely-missing-executable"
    ));
}

#[test]
fn started_process_exceeding_timeout_is_killed_and_marked_timed_out() {
    let workspace = tempfile::tempdir().unwrap();
    let executions = run_verifier(
        workspace.path(),
        &recipe(executable_step(
            "timeout",
            vec![
                "--ignored".into(),
                "--exact".into(),
                "verifier_child_sleeps".into(),
                "--nocapture".into(),
            ],
            1,
        )),
    )
    .unwrap();

    let evidence = &executions[0].evidence;
    assert!(!evidence.passed);
    assert!(evidence.timed_out);
}

#[test]
fn large_output_is_drained_without_deadlock_and_normalized_to_digests() {
    let workspace = tempfile::tempdir().unwrap();
    let executions = run_verifier(
        workspace.path(),
        &recipe(executable_step(
            "large-output",
            vec![
                "--ignored".into(),
                "--exact".into(),
                "verifier_child_large_output".into(),
                "--nocapture".into(),
            ],
            10,
        )),
    )
    .unwrap();

    let evidence = &executions[0].evidence;
    assert!(evidence.passed);
    assert!(!evidence.timed_out);
    assert_eq!(evidence.stdout_sha256.len(), 64);
    assert_eq!(evidence.stderr_sha256.len(), 64);
}

#[test]
fn verifier_working_directory_must_remain_inside_workspace() {
    let workspace = tempfile::tempdir().unwrap();
    let step = VerifierStep {
        working_directory: "../outside".into(),
        ..executable_step("escape", vec!["--list".into()], 10)
    };
    assert!(matches!(
        run_verifier(workspace.path(), &recipe(step)),
        Err(RunnerError::UnsafeOverlayDestination(_))
    ));
}

#[test]
#[ignore = "subprocess fixture invoked by timeout test"]
fn verifier_child_sleeps() {
    thread::sleep(Duration::from_secs(3));
}

#[test]
#[ignore = "subprocess fixture invoked by large-output test"]
fn verifier_child_large_output() {
    let bytes = vec![b'x'; 256 * 1024];
    io::stdout().write_all(&bytes).unwrap();
    io::stdout().flush().unwrap();
}
