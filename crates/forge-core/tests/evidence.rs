//! End-to-end evidence/provenance tests.
//!
//! These prove that an action can be reconstructed from its evidence and that
//! the evidence fingerprint verifies. Trusted execution behavior is exercised
//! separately by the execution-kernel tests; provenance tests do not mint
//! execution authority merely to obtain a fixture.

use chrono::Utc;
use forge_core::{
    record_from, ActionType, AgentAction, Capability, EvidenceStore, ExecutionResult,
    ExecutionStatus, PolicyOutcome, RiskLevel,
};
use serde_json::json;

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

fn successful_git_result(action_id: &str) -> ExecutionResult {
    ExecutionResult {
        action_id: action_id.to_string(),
        status: ExecutionStatus::Succeeded,
        started_at: Utc::now(),
        completed_at: Utc::now(),
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        artifacts: vec![],
        verification: Some(json!({
            "branch": "main",
            "entries": [],
            "clean": true,
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

    let result = successful_git_result(&action.id);

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
    // The fixture preserves the Git status evidence shape for reconstruction.
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
