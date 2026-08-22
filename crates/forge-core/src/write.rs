//! The `write_file` executor — controlled, atomic file mutation.
//!
//! The execution path is deliberate and layered:
//!
//! ```text
//! AgentAction
//!   → validate_action        (structural invariants)
//!   → capability check       (is the `write_file` capability granted?)
//!   → enforce_policy         (risk + trusted authorization grant)
//!   → workspace.resolve_path (containment + symlink/traversal defense)
//!   → proposed change        (validate payload: path + content)
//!   → diff                   (before/after unified diff)
//!   → atomic write           (temp file + rename; never partial)
//!   → evidence               (before/after hashes + new hash + diff)
//! ```

use std::path::Path;

use chrono::Utc;

use crate::action::AgentAction;
use crate::error::ExecutionError;
use crate::evidence::{sha256_hex, ExecutionResult, ExecutionStatus};
use crate::patch::generate_diff;
use crate::policy::{enforce_policy, has_required_capability, AuthorizationGrant};
use crate::workspace::{PathResolution, Workspace};

const PATH_FIELD: &str = "path";
const CONTENT_FIELD: &str = "content";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
    DryRun,
    Atomic,
}

impl WriteMode {
    pub fn persists(self) -> bool {
        matches!(self, WriteMode::Atomic)
    }
}

#[derive(Debug, Clone)]
pub struct WriteOutcome {
    pub path: String,
    pub before_sha256: Option<String>,
    pub after_sha256: String,
    pub diff: Option<String>,
    pub created: bool,
}

/// Backward-compatible write entry point. No approval grant is implied.
pub fn write_file(
    action: &AgentAction,
    workspace: &Workspace,
    mode: WriteMode,
) -> Result<ExecutionResult, ExecutionError> {
    write_file_authorized(action, workspace, mode, &AuthorizationGrant::none())
}

/// Trusted write entry point used by the execution envelope path.
pub(crate) fn write_file_authorized(
    action: &AgentAction,
    workspace: &Workspace,
    mode: WriteMode,
    grant: &AuthorizationGrant,
) -> Result<ExecutionResult, ExecutionError> {
    let started_at = Utc::now();

    enforce_policy(action, grant)?;

    if !has_required_capability(action) {
        return Err(ExecutionError::CapabilityDenied);
    }

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
    let content_value = action
        .payload
        .get(CONTENT_FIELD)
        .ok_or(ExecutionError::MissingPayloadField(CONTENT_FIELD))?;
    let content = content_value
        .as_str()
        .ok_or(ExecutionError::PayloadFieldNotString(CONTENT_FIELD))?;
    let raw_path = Path::new(path_str);

    let resolved = match workspace.resolve_path(raw_path) {
        PathResolution::Allowed(p) => p,
        PathResolution::Denied(p) => {
            return Err(ExecutionError::PathOutsideWorkspace(p));
        }
        PathResolution::Traversal(p) => {
            return Err(ExecutionError::PathTraversal(p));
        }
        PathResolution::Invalid(_) => {
            return Err(ExecutionError::InvalidPath(raw_path.to_path_buf()));
        }
    };

    if content.len() as u64 > workspace.max_bytes() {
        return Err(ExecutionError::OversizedFile(
            resolved.clone(),
            workspace.max_bytes(),
        ));
    }

    let before = std::fs::read(&resolved).ok();
    let before_sha256 = before.as_deref().map(sha256_hex);
    let before_lines: Vec<String> = before
        .as_deref()
        .map(|b| {
            String::from_utf8_lossy(b)
                .lines()
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();
    let after_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let diff = generate_diff(&before_lines, &after_lines);
    let after_sha256 = sha256_hex(content.as_bytes());

    if mode == WriteMode::DryRun {
        return Ok(build_result(
            action,
            ExecutionStatus::Accepted,
            &resolved,
            before_sha256,
            after_sha256,
            diff,
            before.is_none(),
            started_at,
        ));
    }

    atomic_write(&resolved, content.as_bytes())?;

    Ok(build_result(
        action,
        ExecutionStatus::Succeeded,
        &resolved,
        before_sha256,
        after_sha256,
        diff,
        before.is_none(),
        started_at,
    ))
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ExecutionError> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let tmp = dir.join(format!(".{file_name}.forge-tmp-{}", std::process::id()));

    let write_result = (|| -> std::io::Result<()> {
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, path)
    })();

    if let Err(e) = write_result {
        let _ = std::fs::remove_file(&tmp);
        return Err(ExecutionError::Io(path.to_path_buf(), e));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_result(
    action: &AgentAction,
    status: ExecutionStatus,
    path: &Path,
    before_sha256: Option<String>,
    after_sha256: String,
    diff: Option<String>,
    created: bool,
    started_at: chrono::DateTime<Utc>,
) -> ExecutionResult {
    let verification = serde_json::json!({
        "path": path.display().to_string(),
        "before_sha256": before_sha256,
        "after_sha256": after_sha256,
        "diff": diff,
        "created": created,
    });
    let completed_at = Utc::now();
    ExecutionResult {
        action_id: action.id.clone(),
        status,
        started_at,
        completed_at,
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        artifacts: vec![path.display().to_string()],
        verification: Some(verification),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ActionType, AgentAction, Capability, RiskLevel};
    use serde_json::json;

    fn base_action() -> AgentAction {
        AgentAction {
            id: "action-1".to_string(),
            task_id: "task-1".to_string(),
            agent_id: "agent-1".to_string(),
            action_type: ActionType::WriteFile,
            reason: "update config".to_string(),
            risk: RiskLevel::Low,
            capabilities: vec![Capability::WriteFile],
            payload: json!({ "path": "a.txt", "content": "new" }),
            expected: json!({}),
        }
    }

    fn action(path: &str, content: &str) -> AgentAction {
        let mut a = base_action();
        a.payload = json!({ "path": path, "content": content });
        a
    }

    #[test]
    fn atomic_write_creates_and_replaces() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"old").unwrap();
        let result = write_file(&action("a.txt", "new"), &ws, WriteMode::Atomic).unwrap();
        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "new"
        );
    }

    #[test]
    fn dry_run_does_not_touch_filesystem() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"old").unwrap();
        let result = write_file(&action("a.txt", "new"), &ws, WriteMode::DryRun).unwrap();
        assert_eq!(result.status, ExecutionStatus::Accepted);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "old"
        );
    }

    #[test]
    fn denied_capability_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let mut a = action("a.txt", "new");
        a.capabilities = vec![];
        let err = write_file(&a, &ws, WriteMode::Atomic).unwrap_err();
        assert!(matches!(err, ExecutionError::CapabilityDenied));
    }

    #[test]
    fn traversal_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let err = write_file(&action("../escape.txt", "x"), &ws, WriteMode::Atomic).unwrap_err();
        assert!(matches!(err, ExecutionError::PathTraversal(_)));
    }

    #[test]
    fn unauthorized_path_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("ws");
        std::fs::create_dir_all(&root).unwrap();
        let ws = Workspace::new(&root, 4096).unwrap();
        let outside = dir.path().join("secret.txt").display().to_string();
        let err = write_file(&action(&outside, "x"), &ws, WriteMode::Atomic).unwrap_err();
        assert!(matches!(err, ExecutionError::PathOutsideWorkspace(_)));
    }

    #[test]
    fn oversized_content_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4).unwrap();
        let err =
            write_file(&action("a.txt", "toolongcontent"), &ws, WriteMode::Atomic).unwrap_err();
        assert!(matches!(err, ExecutionError::OversizedFile(_, 4)));
    }

    #[test]
    fn missing_content_field_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let mut a = base_action();
        a.payload = json!({ "path": "a.txt" });
        let err = write_file(&a, &ws, WriteMode::Atomic).unwrap_err();
        assert!(matches!(
            err,
            ExecutionError::MissingPayloadField("content")
        ));
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
            let err = write_file(&action("link.txt", "x"), &ws, WriteMode::Atomic).unwrap_err();
            assert!(matches!(
                err,
                ExecutionError::SymlinkEscape(_) | ExecutionError::PathOutsideWorkspace(_)
            ));
        }
    }

    #[test]
    fn approved_by_risk_policy_required() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let mut a = action("a.txt", "x");
        a.risk = RiskLevel::High;
        let err = write_file(&a, &ws, WriteMode::Atomic).unwrap_err();
        assert!(matches!(err, ExecutionError::RequiresApproval));
    }

    #[test]
    fn trusted_grant_allows_approved_high_risk_write() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let mut a = action("a.txt", "x");
        a.risk = RiskLevel::High;
        let grant = AuthorizationGrant::approved("approval-1");
        let result = write_file_authorized(&a, &ws, WriteMode::Atomic, &grant).unwrap();
        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "x"
        );
    }
}
