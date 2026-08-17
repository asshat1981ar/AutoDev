use std::fs;
use std::path::Path;
use std::process::Command;

use autodev_eval::{changed_paths, materialize_checkout, RunnerError};

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

fn fixture_repo() -> (tempfile::TempDir, String, String) {
    let repo = tempfile::tempdir().unwrap();
    let status = Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(repo.path())
        .status()
        .unwrap();
    assert!(status.success());
    git(repo.path(), &["config", "user.name", "AutoDev Eval"]);
    git(
        repo.path(),
        &["config", "user.email", "eval@autodev.invalid"],
    );

    fs::write(repo.path().join("tracked.txt"), "one\n").unwrap();
    git(repo.path(), &["add", "tracked.txt"]);
    git(repo.path(), &["commit", "-q", "-m", "first"]);
    let first = git(repo.path(), &["rev-parse", "HEAD"]);

    fs::write(repo.path().join("tracked.txt"), "two\n").unwrap();
    fs::write(repo.path().join("second.txt"), "second\n").unwrap();
    git(repo.path(), &["add", "."]);
    git(repo.path(), &["commit", "-q", "-m", "second"]);
    let second = git(repo.path(), &["rev-parse", "HEAD"]);
    (repo, first, second)
}

#[test]
fn materialized_checkout_is_detached_at_exact_requested_sha_and_clean() {
    let (source, first, second) = fixture_repo();
    let source_head_before = git(source.path(), &["rev-parse", "HEAD"]);
    let source_status_before = git(source.path(), &["status", "--porcelain=v1"]);
    assert_eq!(source_head_before, second);

    let checkout = materialize_checkout(source.path(), &first).unwrap();
    assert_eq!(git(checkout.path(), &["rev-parse", "HEAD"]), first);
    assert_eq!(git(checkout.path(), &["symbolic-ref", "-q", "HEAD"]), "");
    assert!(changed_paths(checkout.path()).unwrap().is_empty());

    assert_eq!(git(source.path(), &["rev-parse", "HEAD"]), source_head_before);
    assert_eq!(
        git(source.path(), &["status", "--porcelain=v1"]),
        source_status_before
    );
}

#[test]
fn changed_paths_returns_modified_and_untracked_paths_sorted_and_deduplicated() {
    let (source, _, second) = fixture_repo();
    let checkout = materialize_checkout(source.path(), &second).unwrap();
    fs::write(checkout.path().join("tracked.txt"), "modified\n").unwrap();
    fs::write(checkout.path().join("zeta.txt"), "z\n").unwrap();
    fs::create_dir_all(checkout.path().join("nested")).unwrap();
    fs::write(checkout.path().join("nested/alpha.txt"), "a\n").unwrap();

    assert_eq!(
        changed_paths(checkout.path()).unwrap(),
        vec![
            "nested/alpha.txt".to_string(),
            "tracked.txt".to_string(),
            "zeta.txt".to_string(),
        ]
    );
}

#[test]
fn unknown_sha_is_a_typed_git_error() {
    let (source, _, _) = fixture_repo();
    let missing = "0000000000000000000000000000000000000000";
    assert!(matches!(
        materialize_checkout(source.path(), missing),
        Err(RunnerError::Git(_))
    ));
}
