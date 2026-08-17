//! End-to-end evidence/provenance tests.
//!
//! These prove that an action can be reconstructed from its evidence and that
//! the evidence fingerprint verifies. Trusted execution behavior is exercised
//! separately by the execution-kernel tests; provenance tests do not mint
//! execution authority merely to obtain a fixture.

use chrono::Utc;
use forge_core::{
    record_from, ActionType, AgentAction, Capability, EvidenceStore, ExecutableAction,
    ExecutionResult, ExecutionStatus, PolicyOutcome, RiskLevel, Workspace,
};
use serde_json::json;
use std::process::Command;

fn init_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_str().unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.email", "test@example.com"],
        vec!["config", "user.name", "Test"],
    ] {
        assert!(Command::new("git")
            .arg("-C")
            .arg(root)
            .args(&args)
            .status()
            .unwrap()
            .success());
    }
    std::fs::write(dir.path().join("base.txt"), b"base").unwrap();
    assert!(Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["add", "-A"])
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["commit", "-q", "-m", "init"])
        .status()
        .unwrap()
        .success());
    dir
}

fn successful_read_result(action_id: &str, path: &str, content: &str) -> ExecutionResult {
    ExecutionResult {
        action_id: action_id.to_string(),
        status: ExecutionStatus::Succeeded,
        started_at: Utc::now(),
        completed_at: Utc::now(),
        exit_code: None,
        stdout: content.to_string(),
        stderr: String::new(),
        artifacts: vec![path.to_string()],
        verification: Some(json!({
            "path": path,
            "sha256": "fixture-sha256",
            "size": content.len(),
        })),
        error: None,
    }
}

#[test]
fn read_action_is_reconstructed_from_evidence() {
    let action = AgentAction {
        id: "a-read-1".to_string(),
        task_id: "t1".to_string(),
        agent_id: "g1".to_string(),
        action_type: ActionType::ReadFile,
        reason: "inspect".to_string(),
        risk: RiskLevel::Low,
        capabilities: vec![Capability::ReadFile],
        payload: json!({ "path": "a.txt" }),
        expected: json!({}),
    };

    let result = successful_read_result(&action.id, "a.txt", "hello");

    let mut store = EvidenceStore::new();
    let evidence = store.insert(record_from(
        "rec-read-1",
        &action,
        PolicyOutcome::Allow,
        &result,
        vec![],
    ));

    // The evidence fingerprint verifies.
    assert!(evidence.verify());

    // Reconstruct the action from the evidence.
    let record = store.by_action_id("a-read-1").unwrap();
    let reconstructed: AgentAction = serde_json::from_value(record.record.action.clone()).unwrap();
    assert_eq!(reconstructed, action);
    assert_eq!(record.record.status, ExecutionStatus::Succeeded);
    assert_eq!(record.record.task_id, "t1");
    assert_eq!(record.record.agent_id, "g1");
}

#[test]
fn git_action_is_reconstructed_from_evidence() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let action = AgentAction {
        id: "a-git-1".to_string(),
        task_id: "t2".to_string(),
        agent_id: "g2".to_string(),
        action_type: ActionType::Git,
        reason: "git status".to_string(),
        risk: RiskLevel::Low,
        capabilities: vec![Capability::Git],
        payload: json!({ "operation": "status" }),
        expected: json!({}),
    };

    let result = forge_core::execute(&ExecutableAction::new(action.clone(), ws)).unwrap();

    let mut store = EvidenceStore::new();
    let evidence = store.insert(record_from(
        "rec-git-1",
        &action,
        PolicyOutcome::Allow,
        &result,
        vec![],
    ));
    assert!(evidence.verify());

    let record = store.by_action_id("a-git-1").unwrap();
    let reconstructed: AgentAction = serde_json::from_value(record.record.action.clone()).unwrap();
    assert_eq!(reconstructed, action);
    // The git status verification captured the branch.
    let v = record.record.verification.as_ref().unwrap();
    assert!(v["branch"].is_string());
}

#[test]
fn two_actions_are_traceable_by_chain() {
    let mut store = EvidenceStore::new();
    for i in 0..2 {
        let action = AgentAction {
            id: format!("a-{i}"),
            task_id: "t3".to_string(),
            agent_id: "g3".to_string(),
            action_type: ActionType::ReadFile,
            reason: "inspect".to_string(),
            risk: RiskLevel::Low,
            capabilities: vec![Capability::ReadFile],
            payload: json!({ "path": "a.txt" }),
            expected: json!({}),
        };
        let result = successful_read_result(&action.id, "a.txt", "hello");
        store.insert(record_from(
            &format!("rec-{i}"),
            &action,
            PolicyOutcome::Allow,
            &result,
            vec![],
        ));
    }

    assert_eq!(store.len(), 2);
    // All records share the same task and agent -> traceable chain.
    for e in store.records() {
        assert_eq!(e.record.task_id, "t3");
        assert_eq!(e.record.agent_id, "g3");
        assert!(e.verify());
    }
}
