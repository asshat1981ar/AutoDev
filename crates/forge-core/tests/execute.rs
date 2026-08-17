//! Integration tests for the full execution pipeline via `forge_core::execute`.

use std::path::Path;

use forge_core::{
    ActionType, AgentAction, Capability, ExecutableAction, ExecutionError, RiskLevel, Workspace,
};
use serde_json::json;

fn action(path: &str) -> AgentAction {
    AgentAction {
        id: "action-1".to_string(),
        task_id: "task-1".to_string(),
        agent_id: "agent-1".to_string(),
        action_type: ActionType::ReadFile,
        reason: "inspect source".to_string(),
        risk: RiskLevel::Low,
        capabilities: vec![Capability::ReadFile],
        payload: json!({ "path": path }),
        expected: json!({}),
    }
}

#[test]
fn public_execute_does_not_promote_requested_read_capability() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    std::fs::write(dir.path().join("hello.txt"), b"hello world").unwrap();

    let err = forge_core::execute(&ExecutableAction::new(action("hello.txt"), ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::CapabilityDenied));
}

#[test]
fn execute_unsupported_action_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut a = action("a.txt");
    a.action_type = ActionType::Mcp; // not yet implemented
    let exec = ExecutableAction::new(a, ws);
    let err = forge_core::execute(&exec).unwrap_err();
    assert!(matches!(err, ExecutionError::UnsupportedAction(_)));
}

#[test]
fn execute_process_is_fail_closed_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let mut a = action("a.txt");
    a.action_type = ActionType::Execute;
    a.capabilities = vec![Capability::Execute];
    a.payload = json!({ "command": "echo", "args": ["hello"], "timeout_secs": 5 });
    let exec = ExecutableAction::new(a, ws);
    let err = forge_core::execute(&exec).unwrap_err();
    assert!(matches!(err, ExecutionError::ProcessSandboxRequired));
}

#[test]
fn public_read_denial_is_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    std::fs::write(dir.path().join("d.txt"), b"deterministic").unwrap();

    let first =
        forge_core::execute(&ExecutableAction::new(action("d.txt"), ws.clone())).unwrap_err();
    let second = forge_core::execute(&ExecutableAction::new(action("d.txt"), ws)).unwrap_err();
    assert!(matches!(first, ExecutionError::CapabilityDenied));
    assert!(matches!(second, ExecutionError::CapabilityDenied));
}

#[test]
fn dry_run_does_not_touch_filesystem() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let result = forge_core::dry_run(&action("missing.txt"));
    assert!(
        result.is_ok(),
        "dry_run should not require the file to exist"
    );
    let _ = ws;
}

#[test]
fn workspace_rejects_nonexistent_root() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join(Path::new("nonexistent"));
    let error = Workspace::new(&root, 1024).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn public_execute_does_not_promote_requested_write_capability() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    std::fs::write(dir.path().join("c.txt"), b"old").unwrap();

    let mut a = action("c.txt");
    a.action_type = ActionType::WriteFile;
    a.capabilities = vec![Capability::WriteFile];
    a.payload = json!({ "path": "c.txt", "content": "new" });

    let err = forge_core::execute(&ExecutableAction::new(a, ws)).unwrap_err();
    assert!(matches!(err, ExecutionError::CapabilityDenied));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("c.txt")).unwrap(),
        "old"
    );
}

#[test]
fn execute_patches_a_file_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    std::fs::write(dir.path().join("p.txt"), b"one\ntwo\nthree\n").unwrap();

    let mut a = action("p.txt");
    a.action_type = ActionType::PatchFile;
    a.capabilities = vec![Capability::PatchFile];
    a.payload = json!({
        "path": "p.txt",
        "patch": "--- a/p.txt\n+++ b/p.txt\n@@ -1,2 +1,2 @@\n one\n-two\n+2nd\n"
    });

    let result = forge_core::execute(&ExecutableAction::new(a, ws)).unwrap();
    assert_eq!(result.status, forge_core::ExecutionStatus::Succeeded);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("p.txt")).unwrap(),
        "one\n2nd\nthree\n"
    );
    let v = result.verification.unwrap();
    assert_eq!(v["applied_hunks"], 1);
}
