//! The `read_file` executor — bounded, workspace-confined file access.

use chrono::Utc;

use crate::action::AgentAction;
use crate::authority::ExecutionAuthority;
use crate::error::ExecutionError;
use crate::evidence::{sha256_hex, ExecutionResult, ExecutionStatus, ReadMetadata};
use crate::policy::{enforce_policy, has_required_capability};
use crate::workspace::{PathResolution, Workspace};

const PATH_FIELD: &str = "path";

/// Backward-compatible read entry point. No approval grant is implied.
pub fn read_file(
    action: &AgentAction,
    workspace: &Workspace,
) -> Result<ExecutionResult, ExecutionError> {
    read_file_authorized(action, workspace, &ExecutionAuthority::none())
}

/// Trusted read entry point used by the execution-envelope path.
pub(crate) fn read_file_authorized(
    action: &AgentAction,
    workspace: &Workspace,
    authority: &ExecutionAuthority,
) -> Result<ExecutionResult, ExecutionError> {
    let started_at = Utc::now();

    enforce_policy(action, authority)?;
    if !has_required_capability(action, authority) {
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
    let raw_path = std::path::Path::new(path_str);

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

fn is_symlink_escape(raw: &std::path::Path) -> bool {
    !raw.is_absolute()
}

fn read_resolved(
    path: &std::path::Path,
    max_bytes: u64,
    started_at: chrono::DateTime<Utc>,
) -> Result<ExecutionResult, ExecutionError> {
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

    let size = metadata.len();
    if size > max_bytes {
        return Err(ExecutionError::OversizedFile(path.to_path_buf(), max_bytes));
    }

    let bytes = std::fs::read(path).map_err(|e| ExecutionError::Io(path.to_path_buf(), e))?;
    let sha256 = sha256_hex(&bytes);
    let content =
        String::from_utf8(bytes).map_err(|_| ExecutionError::InvalidUtf8(path.to_path_buf()))?;
    let modified_at = metadata.modified().ok().map(chrono::DateTime::from);
    let verification = ReadMetadata {
        path: path.display().to_string(),
        sha256,
        size,
        modified_at,
    };

    Ok(ExecutionResult {
        action_id: String::new(),
        status: ExecutionStatus::Succeeded,
        started_at,
        completed_at: Utc::now(),
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
    use crate::authority::GrantedCapability;
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
        let mut action = base_action();
        action.payload = json!({ "path": path });
        action
    }

    fn read_allowed(
        action: &AgentAction,
        workspace: &Workspace,
    ) -> Result<ExecutionResult, ExecutionError> {
        read_file_authorized(
            action,
            workspace,
            &ExecutionAuthority::granted(vec![GrantedCapability::ReadFile]),
        )
    }

    #[test]
    fn successful_read_returns_content_and_hash() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"hello").unwrap();
        let result = read_allowed(&action_with_path("a.txt"), &ws).unwrap();
        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(result.stdout, "hello");
        let verification = result.verification.unwrap();
        assert_eq!(verification["sha256"], sha256_hex(b"hello"));
        assert_eq!(verification["size"], 5);
    }

    #[test]
    fn missing_file_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let err = read_allowed(&action_with_path("missing.txt"), &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::FileNotFound(_)));
    }

    #[test]
    fn directory_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::create_dir_all(dir.path().join("sub")).unwrap();
        let err = read_allowed(&action_with_path("sub"), &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::IsDirectory(_)));
    }

    #[test]
    fn oversized_file_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4).unwrap();
        std::fs::write(dir.path().join("big.txt"), b"hello").unwrap();
        let err = read_allowed(&action_with_path("big.txt"), &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::OversizedFile(_, 4)));
    }

    #[test]
    fn traversal_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let err = read_allowed(&action_with_path("../outside.txt"), &ws).unwrap_err();
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
        let err = read_allowed(&action_with_path(&outside.display().to_string()), &ws).unwrap_err();
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
            let err = read_allowed(&action_with_path("link.txt"), &ws).unwrap_err();
            assert!(matches!(err, ExecutionError::SymlinkEscape(_)));
        }
    }

    #[test]
    fn denied_capability_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"hello").unwrap();
        let mut action = action_with_path("a.txt");
        action.capabilities.clear();
        let err = read_file(&action, &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::CapabilityDenied));
    }

    #[test]
    fn missing_path_field_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let mut action = base_action();
        action.payload = json!({});
        let err = read_allowed(&action, &ws).unwrap_err();
        assert!(matches!(err, ExecutionError::MissingPayloadField("path")));
    }

    #[test]
    fn high_risk_read_requires_trusted_grant() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"hello").unwrap();
        let mut action = action_with_path("a.txt");
        action.risk = RiskLevel::High;
        assert!(matches!(
            read_file(&action, &ws).unwrap_err(),
            ExecutionError::RequiresApproval
        ));
        let result = read_file_authorized(
            &action,
            &ws,
            &ExecutionAuthority::with_approval(vec![GrantedCapability::ReadFile], "approval-1"),
        )
        .unwrap();
        assert_eq!(result.stdout, "hello");
    }
}
