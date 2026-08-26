use std::path::PathBuf;

use autodev_eval::{load_corpus, smoke_fixture};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn corpus_contains_exactly_five_unique_frozen_tasks() {
    let corpus = load_corpus(fixture_dir()).unwrap();
    assert_eq!(corpus.len(), 5);
    assert!(corpus.iter().all(|fixture| fixture.task.validate().is_ok()));

    let ids: Vec<&str> = corpus
        .iter()
        .map(|fixture| fixture.task.id.as_str())
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
#[ignore = "requires full local AutoDev history and Android/JDK/Node toolchains"]
fn historical_reference_states_are_distinguished_from_base_states() {
    let source_repo = std::env::var("AUTODEV_EVAL_SOURCE_REPO")
        .expect("AUTODEV_EVAL_SOURCE_REPO must point at a full AutoDev checkout");
    let corpus = load_corpus(fixture_dir()).unwrap();

    for fixture in corpus {
        let result = smoke_fixture(&fixture, source_repo.as_ref(), &crate_root()).unwrap();
        assert!(
            !result.base_passed,
            "{} base state unexpectedly passed — verifier evidence: {}",
            result.task_id, result.base_detail
        );
        assert!(
            result.reference_passed,
            "{} accepted/reference state failed — verifier evidence: {}. \
             If exit_code is 127/1 with a near-immediate elapsed_ms, this is \
             environmental (the pinned reference revision needs JDK 17 and \
             Android SDK 35 to assemble the debug APK, as provisioned in CI), \
             not an eval regression",
            result.task_id, result.reference_detail
        );
    }
}
