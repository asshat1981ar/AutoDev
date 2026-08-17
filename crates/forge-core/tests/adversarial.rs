//! Adversarial security test suite for ForgeCore.
//!
//! This integration test suite attempts to break the workspace confinement,
//! policy evaluation, and resource limits of the ForgeCore execution kernel.

use forge_core::{
    action::{ActionType, AgentAction, Capability, RiskLevel},
    error::ExecutionError,
    patch_exec::{patch_file, PatchMode},
    read::read_file,
    workspace::{PathResolution, Workspace},
};
use serde_json::json;

fn base_action(
    action_type: ActionType,
    payload: serde_json::Value,
    caps: Vec<Capability>,
) -> AgentAction {
    AgentAction {
        id: "test-adv-1".into(),
        task_id: "task-adv-1".into(),
        agent_id: "malicious-agent".into(),
        action_type,
        reason: "adversarial test".into(),
        risk: RiskLevel::Low, // Default to low; policy should enforce minimums if needed
        capabilities: caps,
        payload,
        expected: json!({}),
    }
}

// ============================================================================
// 1. Workspace Path Resolution (Traversal & Escape)
// ============================================================================

#[test]
fn adversarial_traversal_deep() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let action = base_action(
        ActionType::ReadFile,
        json!({ "path": "../../../../etc/passwd" }),
        vec![Capability::ReadFile],
    );

    let resolution = ws.resolve_path(std::path::Path::new("../../../../etc/passwd"));
    assert!(matches!(
        resolution,
        PathResolution::Invalid(_) | PathResolution::Denied(_)
    ));

    let err = read_file(&action, &ws).unwrap_err();
    assert!(matches!(err, ExecutionError::CapabilityDenied));
}

#[test]
fn adversarial_absolute_path_escape() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let action = base_action(
        ActionType::ReadFile,
        json!({ "path": "/etc/passwd" }),
        vec![Capability::ReadFile],
    );

    let resolution = ws.resolve_path(std::path::Path::new("/etc/passwd"));
    assert!(matches!(resolution, PathResolution::Denied(_)));

    let err = read_file(&action, &ws).unwrap_err();
    assert!(matches!(err, ExecutionError::CapabilityDenied));
}

#[test]
#[cfg(unix)]
fn adversarial_symlink_escape_read() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("ws");
    std::fs::create_dir_all(&root).unwrap();
    let outside = dir.path().join("secret.txt");
    std::fs::write(&outside, b"top secret data").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("link.txt")).unwrap();

    let ws = Workspace::new(&root, 4096).unwrap();
    let action = base_action(
        ActionType::ReadFile,
        json!({ "path": "link.txt" }),
        vec![Capability::ReadFile],
    );

    let resolution = ws.resolve_path(std::path::Path::new("link.txt"));
    assert!(matches!(resolution, PathResolution::Denied(_)));

    let err = read_file(&action, &ws).unwrap_err();
    assert!(matches!(err, ExecutionError::CapabilityDenied));
}

#[test]
#[cfg(unix)]
fn adversarial_symlink_escape_patch_write() {
    // Tests write confinement via patch_file since write_file may be internal
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("ws");
    std::fs::create_dir_all(&root).unwrap();
    let outside = dir.path().join("target.txt");
    std::fs::write(&outside, b"original\n").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("link.txt")).unwrap();

    let ws = Workspace::new(&root, 4096).unwrap();
    let patch = "--- a/link.txt\n+++ b/link.txt\n@@ -1 +1 @@\n-original\n+overwritten\n";
    let action = base_action(
        ActionType::PatchFile,
        json!({ "path": "link.txt", "patch": patch }),
        vec![Capability::PatchFile],
    );
    let err = patch_file(&action, &ws, PatchMode::Apply).unwrap_err();
    assert!(matches!(err, ExecutionError::SymlinkEscape(_)));

    // Verify the outside file was NOT modified
    assert_eq!(std::fs::read_to_string(&outside).unwrap(), "original\n");
}

#[test]
#[cfg(unix)]
fn adversarial_toctou_symlink_escape_patch() {
    // Attempt to write to a NEW file inside a symlinked directory.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("ws");
    std::fs::create_dir_all(&root).unwrap();
    let outside = dir.path().join("outside_dir");
    std::fs::create_dir_all(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("symlink_dir")).unwrap();

    let ws = Workspace::new(&root, 4096).unwrap();

    // patch_file will fail to read the non-existent file, but it MUST fail
    // at the path resolution step (SymlinkEscape/PathOutsideWorkspace)
    // BEFORE attempting to read or write.
    let patch = "--- a\n+++ b\n@@ -1 +1 @@\n-x\n+y\n";
    let action = base_action(
        ActionType::PatchFile,
        json!({ "path": "symlink_dir/new_file.txt", "patch": patch }),
        vec![Capability::PatchFile],
    );
    let err = patch_file(&action, &ws, PatchMode::Apply).unwrap_err();

    // It should NOT be FileNotFound, because path resolution must catch the symlink first.
    assert!(!matches!(err, ExecutionError::FileNotFound(_)));
    assert!(matches!(
        err,
        ExecutionError::PathOutsideWorkspace(_) | ExecutionError::SymlinkEscape(_)
    ));

    // Verify the file was NOT created outside
    assert!(!outside.join("new_file.txt").exists());
}

// ============================================================================
// 2. Resource Limits
// ============================================================================

#[test]
fn adversarial_read_oversized_file() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 10).unwrap(); // 10 byte limit
    std::fs::write(
        dir.path().join("big.txt"),
        b"this is way more than ten bytes",
    )
    .unwrap();

    let action = base_action(
        ActionType::ReadFile,
        json!({ "path": "big.txt" }),
        vec![Capability::ReadFile],
    );

    // Public/model-originated intent must never reach resource-sensitive reads.
    // The trusted ReadFile path still exercises OversizedFile in read.rs unit tests.
    let err = read_file(&action, &ws).unwrap_err();
    assert!(matches!(err, ExecutionError::CapabilityDenied));
}

// ============================================================================
// 3. Policy & Capability Bypass
// ============================================================================

#[test]
fn adversarial_capability_stripping() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    std::fs::write(dir.path().join("a.txt"), b"hello").unwrap();

    let action = base_action(ActionType::ReadFile, json!({ "path": "a.txt" }), vec![]); // Stripped capabilities
    let err = read_file(&action, &ws).unwrap_err();
    assert!(matches!(err, ExecutionError::CapabilityDenied));
}

#[test]
fn adversarial_patch_traversal() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::new(dir.path(), 4096).unwrap();
    let action = base_action(
        ActionType::PatchFile,
        json!({ "path": "../escape.txt", "patch": "--- a\n+++ b\n@@ -1 +1 @@\n-x\n" }),
        vec![Capability::PatchFile],
    );
    let err = patch_file(&action, &ws, PatchMode::Apply).unwrap_err();
    assert!(matches!(
        err,
        ExecutionError::PathTraversal(_) | ExecutionError::PathOutsideWorkspace(_)
    ));
}
