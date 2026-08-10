//! The `read_file` executor — the first real ForgeCore operation.
//!
//! The execution path is deliberate and layered:
//!
//! ```text
//! AgentAction
//!   → validate_action        (structural invariants)
//!   → capability check       (Is the `read_file` capability granted?)
//!   → evaluate_policy        (risk → Allow / RequireApproval / Deny)
//!   → workspace.resolve_path (containment + symlink/traversal defense)
//!   → metadata + size gate   (reject missing / directory / oversized)
//!   → bounded read           (never read more than max_bytes)
//!   → evidence               (schema-conformant ExecutionResult + hash)
//! ```
//!
//! This module performs **read-only** filesystem access. It deliberately does
//! not create, modify, or delete files, and does not execute any process.

use chrono::Utc;

use crate::action::AgentAction;
use crate::error::ExecutionError;
use crate::evidence::{sha256_hex, ExecutionResult, ExecutionStatus, ReadMetadata};
use crate::policy::{evaluate_policy, has_required_capability, PolicyDecision};
use crate::workspace::{PathResolution, Workspace};

/// The payload field `read_file` understands.
const PATH_FIELD: &str = "path";

/// Execute a `read_file` action.
///
/// Returns a schema-conformant [`ExecutionResult`]. Policy or workspace
/// failures are surfaced as structured [`ExecutionError`]s.
pub fn read_file(
    action: &AgentAction,
    workspace: &Workspace,
) -> Result<ExecutionResult, ExecutionError> {
    let started_at = Utc::now();

    // 1. Structural validation.
    evaluate_policy(action)?;

    // 2. Capability check: reading requires the `read_file` capability.
    if !has_required_capability(action) {
        return Err(ExecutionError::CapabilityDenied);
    }

    // 3. Risk-based policy: medium/high/critical require approval.
    match evaluate_policy(action)? {
        PolicyDecision::Allow => {}
        PolicyDecision::RequireApproval => return Err(ExecutionError::RequiresApproval),
        PolicyDecision::Deny => return Err(ExecutionError::CapabilityDenied),
    }

    // 4. Extract and validate the payload path.
    if !action.payload.is_object() {
        return Err(ExecutionError::PayloadNotObject);
    }
    let path_value = action
        .payload
        .get(PATH_FIELD)
        .ok_or(ExecutionError::MissingPayloadField(PATH_FIELD))?;
    let path_str = path_value
        .as_str()
        .ok_or(ExecutionError::PayloadFieldNotString(PATH_FIELD))?;
    let raw_path = std::path::Path::new(path_str);

    // 5. Resolve against the workspace (containment + symlink/traversal).
    let resolved = match workspace.resolve_path(raw_path) {
        PathResolution::Allowed(p) => p,
        PathResolution::Denied(p) => {
            if is_symlink_escape(raw_path) {
                return Err(ExecutionError::SymlinkEscape(p));
            }
            return Err(ExecutionError::PathOutsideWorkspace(p));
        }
        PathResolution::Invalid(msg) => {
            if msg.contains("traversal") {
                return Err(ExecutionError::PathTraversal(raw_path.to_path_buf()));
            }
            return Err(ExecutionError::InvalidPath(raw_path.to_path_buf()));
        }
    };

    read_resolved(&resolved, workspace.max_bytes(), started_at)
}

/// Whether a raw path, anchored inside the workspace, escaped via a symlink.
/// A relative raw path that resolved outside the root must have been redirected
/// by a symlink (or the root itself changed).
fn is_symlink_escape(raw: &std::path::Path) -> bool {
    !raw.is_absolute()
}

/// Read a workspace-resolved file, enforcing the size gate and producing
/// evidence.
fn read_resolved(
    path: &std::path::Path,
    max_bytes: u64,
    started_at: chrono::DateTime<Utc>,
) -> Result<ExecutionResult, ExecutionError> {
    // Ensure the target exists and is a regular file (not a directory).
    let metadata = std::fs::metadata(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ExecutionError::FileNotFound(path.to_path_buf())
        } else {
            ExecutionError::Io(path.to_path_buf(), e)
        }
    })?;
    if metadata.is_dir() {
        return Err(ExecutionError::IsDirectory(path.to_path_buf()));
    }

    // Size gate: never read more than the workspace limit.
    let size = metadata.len();
    if size > max_bytes {
        return Err(ExecutionError::OversizedFile(path.to_path_buf(), max_bytes));
    }

    // Bounded read: the file is known to be within the size limit.
    let bytes = std::fs::read(path).map_err(|e| ExecutionError::Io(path.to_path_buf(), e))?;

    let sha256 = sha256_hex(&bytes);

    // Decode to UTF-8 for `stdout`; non-UTF-8 content surfaces a structured
    // error (the hash and metadata are still valid).
    let content = match String::from_utf8(bytes.clone()) {
        Ok(s) => s,
        Err(_) => return Err(ExecutionError::InvalidUtf8(path.to_path_buf())),
    };

    let modified_at = metadata.modified().ok().map(chrono::DateTime::from);

    let verification = ReadMetadata {
        path: path.display().to_string(),
        sha256,
        size,
        modified_at,
    };

    let completed_at = Utc::now();
    Ok(ExecutionResult {
        action_id: String::new(), // the caller fills in the action id
        status: ExecutionStatus::Succeeded,
        started_at,
        completed_at,
        exit_code: None,
        stdout: content,
        stderr: String::new(),
        artifacts: vec![path.display().to_string()],
        verification: Some(serde_json::to_value(verification).unwrap()),
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ActionType, AgentAction, Capability, RiskLevel};
    use crate::evidence::ExecutionStatus;
    use serde_json::json;

    fn base_action() -> AgentAction {
        AgentAction {
            id: "action-1".to_string(),
            task_id: "task-1".to_string(),
            agent_id: "agent-1".to_string(),
            action_type: ActionType::ReadFile,
            reason: "inspect source".to_string(),
            risk: RiskLevel::Low,
            capabilities: vec![Capability::ReadFile],
            payload: json!({ "path": "a.txt" }),
            expected: json!({}),
        }
    }

    fn action_with_path(path: &str) -> AgentAction {
        let mut a = base_action();
        a.payload = json!({ "path": path });
        a
    }

    #[test]
    fn successful_read_returns_content_and_hash() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"hello").unwrap();

        let result = read_file(&action_with_path("a.txt"), &ws).unwrap();
        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(result.stdout, "hello");
        let v = result.verification.unwrap();
        assert_eq!(v["sha256"], sha256_hex(b"hello"));
        assert_eq!(v["size"], 5);
    }

    #[test]
    fn missing_file_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let err = read_file(&action_with_path("missing.txt"), &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::FileNotFound(_)));
    }

    #[test]
    fn directory_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::create_dir_all(dir.path().join("sub")).unwrap();
        let err = read_file(&action_with_path("sub"), &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::IsDirectory(_)));
    }

    #[test]
    fn oversized_file_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4).unwrap();
        std::fs::write(dir.path().join("big.txt"), b"hello").unwrap();
        let err = read_file(&action_with_path("big.txt"), &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::OversizedFile(_, 4)));
    }

    #[test]
    fn traversal_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let err = read_file(&action_with_path("../outside.txt"), &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::PathTraversal(_)));
    }

    #[test]
    fn unauthorized_path_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("ws");
        std::fs::create_dir_all(&root).unwrap();
        let ws = Workspace::new(&root, 4096).unwrap();
        let outside = dir.path().join("secret.txt");
        std::fs::write(&outside, b"secret").unwrap();
        let err = read_file(&action_with_path(&outside.display().to_string()), &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::PathOutsideWorkspace(_)));
    }

    #[test]
    fn symlink_escape_is_rejected() {
        #[cfg(unix)]
        {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("ws");
            std::fs::create_dir_all(&root).unwrap();
            let outside = dir.path().join("secret.txt");
            std::fs::write(&outside, b"secret").unwrap();
            std::os::unix::fs::symlink(&outside, root.join("link.txt")).unwrap();
            let ws = Workspace::new(&root, 4096).unwrap();
            let err = read_file(&action_with_path("link.txt"), &ws).unwrap_err();
            assert!(matches!(err, ExecutionError::SymlinkEscape(_)));
        }
    }

    #[test]
    fn denied_capability_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"hello").unwrap();
        let mut action = action_with_path("a.txt");
        action.capabilities = vec![]; // no read_file capability
        let err = read_file(&action, &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::CapabilityDenied));
    }

    #[test]
    fn missing_path_field_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let mut action = base_action();
        action.payload = json!({});
        let err = read_file(&action, &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::MissingPayloadField("path")));
    }
}
