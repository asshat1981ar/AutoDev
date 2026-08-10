//! Integration tests for the Git workspace via `forge_core::execute`.

use forge_core::{
    ActionType, AgentAction, Capability, ExecutableAction, ExecutionError, RiskLevel, Workspace,
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

fn git_action(op: &str) -> AgentAction {
    AgentAction {
        id: "g1".to_string(),
        task_id: "t1".to_string(),
        agent_id: "a1".to_string(),
        action_type: ActionType::Git,
        reason: "git workspace".to_string(),
        risk: RiskLevel::Low,
        capabilities: vec![Capability::Git],
        payload: json!({ "operation": op }),
        expected: json!({}),
    }
}

#[test]
fn git_read_status_via_execute() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let result = forge_core::execute(&ExecutableAction::new(git_action("status"), ws)).unwrap();
    assert_eq!(result.verification.unwrap()["branch"], "master");
}

#[test]
fn git_read_rejected_without_git_capability() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut a = git_action("status");
    a.capabilities = vec![]; // no git capability
    let err = forge_core::execute(&ExecutableAction::new(a, ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::GitCapabilityDenied(_)));
}

#[test]
fn git_mutate_requires_git_write_capability() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut a = git_action("prepare_commit");
    a.payload = json!({ "operation": "prepare_commit", "message": "x", "commit": true });
    // Only the read `git` capability is granted, not `git:write`.
    let err = forge_core::execute(&ExecutableAction::new(a, ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::GitCapabilityDenied(_)));
}

#[test]
#[ignore = "rollback now requires both git:destructive capability AND approval; covered by unit tests"]
fn git_destructive_requires_git_destructive_capability() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut a = git_action("rollback");
    a.payload = json!({ "operation": "rollback", "command": "reset", "to": "HEAD" });
    let err = forge_core::execute(&ExecutableAction::new(a, ws)).unwrap_err();
    // Without the destructive capability the operation is denied by capability.
    assert!(matches!(err, ExecutionError::GitCapabilityDenied(_)));
}

#[test]
fn git_destructive_requires_approval_even_with_capability() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut a = git_action("rollback");
    a.capabilities = vec![Capability::Git, Capability::GitDestructive];
    a.payload = json!({ "operation": "rollback", "command": "checkout" });
    // Has the capability, but no approval -> refused.
    let err = forge_core::execute(&ExecutableAction::new(a, ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::RequiresApproval));
}

#[test]
fn git_destructive_denied_without_capability() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut a = git_action("rollback");
    // Only the read `git` capability is granted, not `git:destructive`.
    a.payload = json!({ "operation": "rollback", "command": "checkout", "approved": true });
    let err = forge_core::execute(&ExecutableAction::new(a, ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::GitCapabilityDenied(_)));
}

#[test]
fn git_checkpoint_with_write_capability() {
    let dir = init_repo();
    std::fs::write(dir.path().join("wip.txt"), b"wip").unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut a = git_action("checkpoint");
    a.capabilities = vec![Capability::Git, Capability::GitWrite];
    a.payload = json!({ "operation": "checkpoint", "message": "wip" });
    let result = forge_core::execute(&ExecutableAction::new(a, ws)).unwrap();
    let v = result.verification.unwrap();
    assert!(v["reference"].is_string());
}

#[test]
fn git_not_a_repository_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let err = forge_core::execute(&ExecutableAction::new(git_action("status"), ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::NotARepository(_)));
}
