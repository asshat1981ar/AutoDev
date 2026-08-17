//! Integration tests for the public Git workspace boundary via `forge_core::execute`.

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
fn public_git_read_does_not_promote_requested_capability() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let err = forge_core::execute(&ExecutableAction::new(git_action("status"), ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::GitCapabilityDenied("git")));
}

#[test]
fn git_read_rejected_without_git_capability() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut action = git_action("status");
    action.capabilities.clear();
    let err = forge_core::execute(&ExecutableAction::new(action, ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::GitCapabilityDenied(_)));
}

#[test]
fn git_mutate_requires_kernel_git_write_authority() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut action = git_action("prepare_commit");
    action.capabilities = vec![Capability::Git, Capability::GitWrite];
    action.payload = json!({ "operation": "prepare_commit", "message": "x", "commit": true });
    let err = forge_core::execute(&ExecutableAction::new(action, ws)).unwrap_err();
    assert!(matches!(
        err,
        ExecutionError::GitCapabilityDenied("git:write")
    ));
}

#[test]
fn requested_destructive_capability_does_not_reach_approval_gate() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut action = git_action("rollback");
    action.capabilities = vec![Capability::Git, Capability::GitDestructive];
    action.payload = json!({ "operation": "rollback", "command": "checkout" });
    let err = forge_core::execute(&ExecutableAction::new(action, ws)).unwrap_err();
    assert!(matches!(
        err,
        ExecutionError::GitCapabilityDenied("git:destructive")
    ));
}

#[test]
fn caller_supplied_git_approval_is_not_authority() {
    let dir = init_repo();
    std::fs::write(dir.path().join("base.txt"), b"changed").unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut action = git_action("rollback");
    action.capabilities = vec![Capability::Git, Capability::GitDestructive];
    action.payload = json!({
        "operation": "rollback",
        "command": "checkout",
        "approved": true
    });

    let err = forge_core::execute(&ExecutableAction::new(action, ws)).unwrap_err();
    assert!(matches!(
        err,
        ExecutionError::GitCapabilityDenied("git:destructive")
    ));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("base.txt")).unwrap(),
        "changed"
    );
}

#[test]
fn trusted_approval_alone_does_not_grant_git_authority() {
    let dir = init_repo();
    std::fs::write(dir.path().join("base.txt"), b"changed").unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut action = git_action("rollback");
    action.capabilities = vec![Capability::Git, Capability::GitDestructive];
    action.payload = json!({ "operation": "rollback", "command": "checkout" });

    let err = forge_core::execute(&ExecutableAction::with_approval(action, ws, "approval-1"))
        .unwrap_err();
    assert!(matches!(
        err,
        ExecutionError::GitCapabilityDenied("git:destructive")
    ));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("base.txt")).unwrap(),
        "changed"
    );
}

#[test]
fn git_destructive_denied_without_capability() {
    let dir = init_repo();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut action = git_action("rollback");
    action.payload = json!({ "operation": "rollback", "command": "checkout", "approved": true });
    let err = forge_core::execute(&ExecutableAction::new(action, ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::GitCapabilityDenied(_)));
}

#[test]
fn requested_git_write_does_not_authorize_checkpoint() {
    let dir = init_repo();
    std::fs::write(dir.path().join("wip.txt"), b"wip").unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut action = git_action("checkpoint");
    action.capabilities = vec![Capability::Git, Capability::GitWrite];
    action.payload = json!({ "operation": "checkpoint", "message": "wip" });
    let err = forge_core::execute(&ExecutableAction::new(action, ws)).unwrap_err();
    assert!(matches!(
        err,
        ExecutionError::GitCapabilityDenied("git:write")
    ));
    assert!(dir.path().join("wip.txt").exists());
}

#[test]
fn authority_denial_precedes_repository_probe() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let err = forge_core::execute(&ExecutableAction::new(git_action("status"), ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::GitCapabilityDenied("git")));
}
