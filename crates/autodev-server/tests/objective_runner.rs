use std::fs;

use autodev_server::{
    FileObjectiveStore, ObjectiveSnapshot, ObjectiveStatus, ObjectiveStore, ObjectiveView,
};
use forge_core::{TaskGraph, VerifiedOrchestratorState};
use tempfile::tempdir;

fn snapshot(id: &str) -> ObjectiveSnapshot {
    ObjectiveSnapshot {
        view: ObjectiveView {
            id: id.to_string(),
            repository: "asshat1981ar/AutoDev".to_string(),
            description: "Persist objective state".to_string(),
            branch: "autodev/persist-objective".to_string(),
            status: ObjectiveStatus::Running,
            current_task_id: Some("t-root".to_string()),
            current_phase: Some("act".to_string()),
            latest_evidence_ref: Some("evidence-1".to_string()),
            blocked_reason: None,
        },
        graph: TaskGraph::single("Persist objective state", "round-trip durable task state"),
        orchestrator: VerifiedOrchestratorState::default(),
    }
}

#[test]
fn file_store_round_trips_objective_snapshot_across_instances() {
    let directory = tempdir().expect("temp directory");
    let store = FileObjectiveStore::new(directory.path());
    let expected = snapshot("objective-1");

    store.put(&expected).expect("persist snapshot");
    assert!(!directory.path().join("objective-1.json.tmp").exists());

    let restarted = FileObjectiveStore::new(directory.path());
    assert_eq!(
        restarted.get("objective-1").expect("load snapshot"),
        Some(expected.clone())
    );
    assert_eq!(
        restarted.load_all().expect("load snapshots"),
        vec![expected]
    );
}

#[test]
fn file_store_ignores_stray_temporary_files() {
    let directory = tempdir().expect("temp directory");
    fs::write(
        directory.path().join("orphan.json.tmp"),
        br#"{"partial": true"#,
    )
    .expect("write partial file");

    let store = FileObjectiveStore::new(directory.path());
    assert!(store.load_all().expect("load snapshots").is_empty());
    assert_eq!(store.get("orphan").expect("load orphan"), None);
}
