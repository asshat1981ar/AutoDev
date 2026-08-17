use std::fs;
use std::path::{Path, PathBuf};

use autodev_eval::{load_corpus, load_fixture, FixtureError};
use serde_json::{json, Value};

fn digest(ch: char) -> String {
    std::iter::repeat_n(ch, 64).collect()
}

fn fixture_json(id: &str, asset_fingerprints: Vec<String>, overlay: Vec<Value>) -> Value {
    json!({
        "task": {
            "id": id,
            "source": {
                "kind": "commit",
                "repository": "asshat1981ar/AutoDev",
                "source_ref": "6df35bf674af8023779f59b6770135dca2895d74",
                "source_url": null
            },
            "base_sha": "5c0adf94d192aef131c96d4cb72ef00e30bf7501",
            "specification": "exercise the curated fixture loader",
            "acceptance_criteria": ["required verifier passes"],
            "verifier": {
                "steps": [{
                    "id": "verify",
                    "program": "cargo",
                    "args": ["test"],
                    "working_directory": "crates",
                    "timeout_seconds": 60,
                    "required": true
                }],
                "asset_fingerprints": asset_fingerprints
            },
            "protected": { "paths": [".autodev-eval/"] },
            "expected_change_scope": ["crates/forge-core/"]
        },
        "verifier_overlay": overlay
    })
}

fn overlay(source: &str, destination: &str, sha256: String) -> Value {
    json!({
        "source_path": source,
        "destination_path": destination,
        "sha256": sha256
    })
}

fn write_json(dir: &Path, name: &str, value: &Value) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    path
}

#[test]
fn valid_fixture_loads_and_validates_embedded_task() {
    let dir = tempfile::tempdir().unwrap();
    let sha = digest('a');
    let path = write_json(
        dir.path(),
        "valid.json",
        &fixture_json(
            "fixture-task",
            vec![sha.clone()],
            vec![overlay(
                "fixture-assets/probe.rs",
                "crates/forge-core/tests/eval_probe.rs",
                sha,
            )],
        ),
    );

    let fixture = load_fixture(&path).unwrap();
    assert_eq!(fixture.task.id, "fixture-task");
    fixture.task.validate().unwrap();
    assert_eq!(fixture.verifier_overlay.len(), 1);
}

#[test]
fn corpus_reads_only_json_and_returns_tasks_sorted_by_id() {
    let dir = tempfile::tempdir().unwrap();
    write_json(
        dir.path(),
        "z.json",
        &fixture_json("zeta-task", vec![], vec![]),
    );
    write_json(
        dir.path(),
        "a.json",
        &fixture_json("alpha-task", vec![], vec![]),
    );
    fs::write(dir.path().join("README.txt"), b"not a fixture").unwrap();

    let corpus = load_corpus(dir.path()).unwrap();
    let ids: Vec<&str> = corpus.iter().map(|fixture| fixture.task.id.as_str()).collect();
    assert_eq!(ids, vec!["alpha-task", "zeta-task"]);
}

#[test]
fn duplicate_task_ids_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    write_json(
        dir.path(),
        "one.json",
        &fixture_json("duplicate-task", vec![], vec![]),
    );
    write_json(
        dir.path(),
        "two.json",
        &fixture_json("duplicate-task", vec![], vec![]),
    );

    assert!(matches!(
        load_corpus(dir.path()),
        Err(FixtureError::DuplicateTaskId(id)) if id == "duplicate-task"
    ));
}

#[test]
fn traversal_and_absolute_overlay_paths_are_rejected() {
    for (source, destination) in [
        ("../secret.rs", "crates/forge-core/tests/probe.rs"),
        ("fixture-assets/probe.rs", "../outside/probe.rs"),
        ("fixture-assets/probe.rs", "/tmp/probe.rs"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let sha = digest('b');
        let path = write_json(
            dir.path(),
            "unsafe.json",
            &fixture_json(
                "unsafe-overlay",
                vec![sha.clone()],
                vec![overlay(source, destination, sha)],
            ),
        );

        assert!(matches!(
            load_fixture(path),
            Err(FixtureError::UnsafeOverlayPath(_))
        ));
    }
}

#[test]
fn overlay_digest_must_be_full_sha256() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_json(
        dir.path(),
        "bad-digest.json",
        &fixture_json(
            "bad-digest",
            vec!["abc".into()],
            vec![overlay(
                "fixture-assets/probe.rs",
                "crates/forge-core/tests/probe.rs",
                "abc".into(),
            )],
        ),
    );

    assert!(matches!(
        load_fixture(path),
        Err(FixtureError::InvalidOverlayDigest(value)) if value == "abc"
    ));
}

#[test]
fn overlay_digests_must_exactly_match_verifier_asset_fingerprints() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_json(
        dir.path(),
        "mismatch.json",
        &fixture_json(
            "mismatch-task",
            vec![digest('a')],
            vec![overlay(
                "fixture-assets/probe.rs",
                "crates/forge-core/tests/probe.rs",
                digest('b'),
            )],
        ),
    );

    assert!(matches!(
        load_fixture(path),
        Err(FixtureError::OverlayFingerprintMismatch(id)) if id == "mismatch-task"
    ));
}

#[test]
fn fingerprints_without_overlays_are_rejected_but_empty_both_is_valid() {
    let dir = tempfile::tempdir().unwrap();
    let mismatch = write_json(
        dir.path(),
        "missing-overlay.json",
        &fixture_json("missing-overlay", vec![digest('c')], vec![]),
    );
    assert!(matches!(
        load_fixture(mismatch),
        Err(FixtureError::OverlayFingerprintMismatch(id)) if id == "missing-overlay"
    ));

    let valid = write_json(
        dir.path(),
        "no-overlay.json",
        &fixture_json("no-overlay", vec![], vec![]),
    );
    load_fixture(valid).unwrap();
}
