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
fn execute_reads_a_file_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    std::fs::write(dir.path().join("hello.txt"), b"hello world").unwrap();

    let exec = ExecutableAction::new(action("hello.txt"), ws);
    let result = forge_core::execute(&exec).unwrap();
    assert_eq!(result.action_id, "action-1");
    assert_eq!(result.stdout, "hello world");
    assert_eq!(result.status, forge_core::ExecutionStatus::Succeeded);
    let v = result.verification.unwrap();
    assert_eq!(v["size"], 11);
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
fn execution_is_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    std::fs::write(dir.path().join("d.txt"), b"deterministic").unwrap();

    let e1 = forge_core::execute(&ExecutableAction::new(action("d.txt"), ws.clone())).unwrap();
    let e2 = forge_core::execute(&ExecutableAction::new(action("d.txt"), ws)).unwrap();
    // Same input -> identical content hash and verification payload.
    assert_eq!(e1.verification, e2.verification);
    assert_eq!(e1.stdout, e2.stdout);
}

#[test]
fn dry_run_does_not_touch_filesystem() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    // The file does not exist, but dry_run must succeed without touching it.
    let result = forge_core::dry_run(&action("missing.txt"));
    assert!(
        result.is_ok(),
        "dry_run should not require the file to exist"
    );
    let _ = ws; // keep ws alive
}

#[test]
fn workspace_rejects_nonexistent_root() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join(Path::new("nonexistent"));
    let error = Workspace::new(&root, 1024).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn execute_writes_a_file_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    std::fs::write(dir.path().join("c.txt"), b"old").unwrap();

    let mut a = action("c.txt");
    a.action_type = ActionType::WriteFile;
    a.capabilities = vec![Capability::WriteFile];
    a.payload = json!({ "path": "c.txt", "content": "new" });

    let result = forge_core::execute(&ExecutableAction::new(a, ws)).unwrap();
    assert_eq!(result.status, forge_core::ExecutionStatus::Succeeded);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("c.txt")).unwrap(),
        "new"
    );
    let v = result.verification.unwrap();
    assert!(v["diff"].is_string());
    assert_eq!(v["created"], false);
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
