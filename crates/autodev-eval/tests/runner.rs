use std::fs;
use std::path::Path;
use std::process::Command;

use autodev_eval::{
    AttemptDriver, AttemptMetadata, EvalFixture, EvaluationRunner, RunnerError, VerifierOverlay,
};
use forge_core::{
    EvalStatus, EvalTask, ProtectedSurface, TaskSource, TaskSourceKind, VerificationRecipe,
    VerifierStep,
};
use sha2::{Digest, Sha256};

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git executable");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn source_repo() -> (tempfile::TempDir, String) {
    let repo = tempfile::tempdir().unwrap();
    assert!(Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(repo.path())
        .status()
        .unwrap()
        .success());
    git(repo.path(), &["config", "user.name", "AutoDev Eval"]);
    git(
        repo.path(),
        &["config", "user.email", "eval@autodev.invalid"],
    );
    fs::write(repo.path().join("target.txt"), "base\n").unwrap();
    git(repo.path(), &["add", "."]);
    git(repo.path(), &["commit", "-q", "-m", "base"]);
    let sha = git(repo.path(), &["rev-parse", "HEAD"]);
    (repo, sha)
}

fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn task(base_sha: &str, asset_fingerprints: Vec<String>) -> EvalTask {
    EvalTask {
        id: "runner-task".into(),
        source: TaskSource {
            kind: TaskSourceKind::Commit,
            repository: "asshat1981ar/AutoDev".into(),
            source_ref: base_sha.into(),
            source_url: None,
        },
        base_sha: base_sha.into(),
        specification: "exercise evaluation runner sequencing".into(),
        acceptance_criteria: vec!["independent verifier passes".into()],
        verifier: VerificationRecipe {
            steps: vec![VerifierStep {
                id: "verify".into(),
                program: std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                args: vec!["--list".into()],
                working_directory: ".".into(),
                timeout_seconds: 10,
                required: true,
            }],
            asset_fingerprints,
        },
        protected: ProtectedSurface {
            paths: vec![".autodev-eval/".into()],
        },
        expected_change_scope: vec!["target.txt".into()],
    }
}

struct WritingDriver {
    path: String,
    contents: Vec<u8>,
}

impl AttemptDriver for WritingDriver {
    fn run(
        &mut self,
        _task: &EvalTask,
        workspace: &Path,
    ) -> Result<AttemptMetadata, RunnerError> {
        let destination = workspace.join(&self.path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(destination, &self.contents)?;
        Ok(AttemptMetadata {
            attempts: 1,
            elapsed_ms: 12,
            tool_calls: Some(1),
            intervention_count: Some(0),
        })
    }
}

struct FailingDriver;

impl AttemptDriver for FailingDriver {
    fn run(
        &mut self,
        _task: &EvalTask,
        _workspace: &Path,
    ) -> Result<AttemptMetadata, RunnerError> {
        Err(RunnerError::Io(std::io::Error::other("driver failed")))
    }
}

#[test]
fn runner_captures_agent_changes_before_overlay_and_reports_collision() {
    let (source, sha) = source_repo();
    let crate_root = tempfile::tempdir().unwrap();
    let hidden = b"hidden verifier-owned bytes\n";
    let asset_source = crate_root.path().join("fixture-assets/probe.txt");
    fs::create_dir_all(asset_source.parent().unwrap()).unwrap();
    fs::write(&asset_source, hidden).unwrap();
    let hidden_digest = digest(hidden);
    let fixture = EvalFixture {
        task: task(&sha, vec![hidden_digest.clone()]),
        verifier_overlay: vec![VerifierOverlay {
            source_path: "fixture-assets/probe.txt".into(),
            destination_path: "hidden/probe.txt".into(),
            sha256: hidden_digest,
        }],
    };

    let driver = WritingDriver {
        path: "hidden/probe.txt".into(),
        contents: b"agent bytes\n".to_vec(),
    };
    let mut runner = EvaluationRunner::new(driver, crate_root.path());
    let outcome = runner.evaluate(&fixture, source.path()).unwrap();

    assert_eq!(outcome.status, EvalStatus::Unsolved);
    assert!(outcome
        .changed_paths
        .iter()
        .any(|path| path == "hidden/probe.txt"));
    assert!(outcome.safety_findings.iter().any(|finding| {
        finding.kind == "verifier_overlay_collision"
            && finding.path.as_deref() == Some("hidden/probe.txt")
    }));
}

#[test]
fn successful_driver_and_verifier_produce_solved_outcome() {
    let (source, sha) = source_repo();
    let fixture = EvalFixture {
        task: task(&sha, vec![]),
        verifier_overlay: vec![],
    };
    let driver = WritingDriver {
        path: "target.txt".into(),
        contents: b"candidate\n".to_vec(),
    };
    let crate_root = tempfile::tempdir().unwrap();
    let mut runner = EvaluationRunner::new(driver, crate_root.path());
    let outcome = runner.evaluate(&fixture, source.path()).unwrap();

    assert_eq!(outcome.status, EvalStatus::Solved);
    assert_eq!(outcome.attempts, 1);
    assert_eq!(outcome.tool_calls, Some(1));
    assert_eq!(outcome.intervention_count, Some(0));
    assert_eq!(outcome.verifier_evidence.len(), 1);
}

#[test]
fn driver_failure_is_an_infrastructure_outcome_not_unsolved() {
    let (source, sha) = source_repo();
    let fixture = EvalFixture {
        task: task(&sha, vec![]),
        verifier_overlay: vec![],
    };
    let crate_root = tempfile::tempdir().unwrap();
    let mut runner = EvaluationRunner::new(FailingDriver, crate_root.path());
    let outcome = runner.evaluate(&fixture, source.path()).unwrap();

    assert_eq!(outcome.status, EvalStatus::InfrastructureFailure);
    assert!(outcome.infrastructure_error.is_some());
    assert_eq!(outcome.attempts, 0);
}
