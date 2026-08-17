//! Adversarial tests for the trust boundary between model intent and kernel authority.

use std::process::Command;

use forge_core::{
    action::AgentAction, execute_git, patch_file, read_file, write_file, ExecutionError, PatchMode,
    Workspace, WriteMode,
};
use serde_json::json;

#[test]
fn serialized_action_cannot_self_grant_capabilities() {
    let forged = json!({
        "id": "forged-action",
        "task_id": "task-1",
        "agent_id": "untrusted-agent",
        "type": "write_file",
        "reason": "attempt capability forgery",
        "risk": "low",
        "capabilities": ["write_file", "approval:critical"],
        "payload": { "path": "src/lib.rs", "content": "forged" },
        "expected": {}
    });

    let parsed = serde_json::from_value::<AgentAction>(forged);

    assert!(
        parsed.is_err(),
        "untrusted serialized intent must not be able to supply execution capabilities"
    );
}

#[test]
fn requested_capabilities_do_not_authorize_write_execution() {
    let dir = tempfile::tempdir().expect("temporary workspace");
    let workspace = Workspace::new(dir.path(), 4096).expect("workspace");
    let requested = json!({
        "id": "requested-write",
        "task_id": "task-1",
        "agent_id": "untrusted-agent",
        "type": "write_file",
        "reason": "request write authority",
        "risk": "low",
        "requested_capabilities": ["write_file"],
        "payload": { "path": "forged.txt", "content": "forged" },
        "expected": {}
    });
    let action: AgentAction = serde_json::from_value(requested).expect("valid intent");

    let error = write_file(&action, &workspace, WriteMode::Atomic)
        .expect_err("requested capability must not authorize execution");

    assert!(matches!(error, ExecutionError::CapabilityDenied));
    assert!(!dir.path().join("forged.txt").exists());
}

#[test]
fn requested_capabilities_do_not_authorize_read_execution() {
    let dir = tempfile::tempdir().expect("temporary workspace");
    let workspace = Workspace::new(dir.path(), 4096).expect("workspace");
    std::fs::write(dir.path().join("secret.txt"), b"secret").expect("fixture");
    let requested = json!({
        "id": "requested-read",
        "task_id": "task-1",
        "agent_id": "untrusted-agent",
        "type": "read_file",
        "reason": "request read authority",
        "risk": "low",
        "requested_capabilities": ["read_file"],
        "payload": { "path": "secret.txt" },
        "expected": {}
    });
    let action: AgentAction = serde_json::from_value(requested).expect("valid intent");

    let error = read_file(&action, &workspace)
        .expect_err("requested capability must not authorize read execution");

    assert!(matches!(error, ExecutionError::CapabilityDenied));
}

#[test]
fn requested_capabilities_do_not_authorize_patch_execution() {
    let dir = tempfile::tempdir().expect("temporary workspace");
    let workspace = Workspace::new(dir.path(), 4096).expect("workspace");
    let target = dir.path().join("target.txt");
    std::fs::write(&target, b"one\ntwo\n").expect("fixture");
    let requested = json!({
        "id": "requested-patch",
        "task_id": "task-1",
        "agent_id": "untrusted-agent",
        "type": "patch_file",
        "reason": "request patch authority",
        "risk": "low",
        "requested_capabilities": ["patch_file"],
        "payload": {
            "path": "target.txt",
            "patch": "--- a/target.txt\n+++ b/target.txt\n@@ -1,2 +1,2 @@\n one\n-two\n+changed\n"
        },
        "expected": {}
    });
    let action: AgentAction = serde_json::from_value(requested).expect("valid intent");

    let error = patch_file(&action, &workspace, PatchMode::Apply)
        .expect_err("requested capability must not authorize patch execution");

    assert!(matches!(error, ExecutionError::CapabilityDenied));
    assert_eq!(
        std::fs::read_to_string(target).expect("target"),
        "one\ntwo\n"
    );
}

#[test]
fn requested_capabilities_do_not_authorize_git_read_execution() {
    let dir = tempfile::tempdir().expect("temporary workspace");
    let init = Command::new("git")
        .arg("-C")
        .arg(dir.path())
        .args(["init", "-q"])
        .status()
        .expect("git init");
    assert!(init.success());
    let workspace = Workspace::new(dir.path(), 4096).expect("workspace");
    let requested = json!({
        "id": "requested-git-read",
        "task_id": "task-1",
        "agent_id": "untrusted-agent",
        "type": "git",
        "reason": "request git read authority",
        "risk": "low",
        "requested_capabilities": ["git"],
        "payload": { "operation": "status" },
        "expected": {}
    });
    let action: AgentAction = serde_json::from_value(requested).expect("valid intent");

    let error = execute_git(&action, &workspace)
        .expect_err("requested capability must not authorize git subprocess execution");

    assert!(matches!(error, ExecutionError::GitCapabilityDenied("git")));
}
