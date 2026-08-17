use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use forge_core::{EvalReport, EvalTaskKey};
use serde_json::{json, Value};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_autodev-eval"))
}

fn run(args: &[&str]) -> Output {
    Command::new(bin()).args(args).output().unwrap()
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn synthetic_repo() -> (tempfile::TempDir, String, String) {
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
    fs::write(repo.path().join("README.md"), "base\n").unwrap();
    git(repo.path(), &["add", "."]);
    git(repo.path(), &["commit", "-q", "-m", "base"]);
    let base = git(repo.path(), &["rev-parse", "HEAD"]);

    fs::write(repo.path().join("feature.txt"), "present\n").unwrap();
    git(repo.path(), &["add", "."]);
    git(repo.path(), &["commit", "-q", "-m", "feature"]);
    let reference = git(repo.path(), &["rev-parse", "HEAD"]);
    (repo, base, reference)
}

fn write_smoke_fixture(dir: &Path, base: &str, reference: &str) {
    let value = json!({
        "task": {
            "id": "synthetic-smoke",
            "source": {
                "kind": "commit",
                "repository": "local/synthetic",
                "source_ref": reference,
                "source_url": null
            },
            "base_sha": base,
            "specification": "add feature.txt",
            "acceptance_criteria": ["feature.txt exists"],
            "verifier": {
                "steps": [{
                    "id": "feature-exists",
                    "program": "test",
                    "args": ["-f", "feature.txt"],
                    "working_directory": ".",
                    "timeout_seconds": 10,
                    "required": true
                }],
                "asset_fingerprints": []
            },
            "protected": {"paths": [".autodev-eval/"]},
            "expected_change_scope": ["feature.txt"]
        },
        "verifier_overlay": []
    });
    fs::write(
        dir.join("synthetic-smoke.json"),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}

fn report(revision: &str, success_bps: u16) -> EvalReport {
    EvalReport {
        revision: revision.into(),
        task_keys: vec![EvalTaskKey {
            task_id: "task-a".into(),
            task_fingerprint: "a".repeat(64),
            verifier_fingerprint: "b".repeat(64),
        }],
        tasks_total: 1,
        tasks_scored: 1,
        tasks_solved: u32::from(success_bps == 10_000),
        success_bps,
        safety_regressions: 0,
        infrastructure_failures: 0,
        total_attempts: 1,
        median_attempts_milli: 1000,
        elapsed_ms: 1,
        tool_calls: Some(1),
        intervention_count: Some(0),
        fingerprint: if revision == "baseline" {
            "c".repeat(64)
        } else {
            "d".repeat(64)
        },
    }
}

#[test]
fn unknown_command_prints_usage_and_exits_two() {
    let output = run(&["unknown"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
}

#[test]
fn validate_prints_deterministic_five_task_summary() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let output = run(&["validate", "--fixtures", fixtures.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["task_count"], 5);
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        vec![
            "android-command-center",
            "architecture-evidence-forge",
            "kmp-rebuild-toolchain",
            "rust-control-plane-secure-webhook",
            "termux-kanban-pty-repair",
        ]
    );
}

#[test]
fn compare_prints_typed_improved_decision_and_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let baseline = dir.path().join("baseline.json");
    let candidate = dir.path().join("candidate.json");
    fs::write(
        &baseline,
        serde_json::to_vec_pretty(&report("baseline", 0)).unwrap(),
    )
    .unwrap();
    fs::write(
        &candidate,
        serde_json::to_vec_pretty(&report("candidate", 10_000)).unwrap(),
    )
    .unwrap();

    let output = run(&[
        "compare",
        "--baseline",
        baseline.to_str().unwrap(),
        "--candidate",
        candidate.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["decision"], "improved");
    assert_eq!(value["success_delta_bps"], 10_000);
}

#[test]
fn smoke_exits_zero_only_when_base_fails_and_reference_passes() {
    let (repo, base, reference) = synthetic_repo();
    let fixtures = tempfile::tempdir().unwrap();
    write_smoke_fixture(fixtures.path(), &base, &reference);

    let output = run(&[
        "smoke",
        "--fixtures",
        fixtures.path().to_str().unwrap(),
        "--source-repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["results"][0]["base_passed"], false);
    assert_eq!(value["results"][0]["reference_passed"], true);
}
