//! The `patch_file` executor — deterministic, workspace-confined patching.

use std::path::Path;

use chrono::Utc;

use crate::action::AgentAction;
use crate::error::ExecutionError;
use crate::evidence::{sha256_hex, ExecutionResult, ExecutionStatus};
use crate::patch::{Patch, PatchResult};
use crate::policy::{
    enforce_policy, has_required_execution_authority, AuthorizationGrant, ExecutionAuthority,
};
use crate::workspace::{PathResolution, Workspace};
use crate::write::atomic_write;

const PATH_FIELD: &str = "path";
const PATCH_FIELD: &str = "patch";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchMode {
    DryRun,
    Apply,
}

/// Safe public patch entry point.
///
/// Model-supplied requested capabilities are intent, never effective authority.
/// Without independently minted kernel authority this entry point fails closed.
pub fn patch_file(
    action: &AgentAction,
    workspace: &Workspace,
    mode: PatchMode,
) -> Result<ExecutionResult, ExecutionError> {
    patch_file_authorized(
        action,
        workspace,
        mode,
        &ExecutionAuthority::deny_all(),
        &AuthorizationGrant::none(),
    )
}

/// Trusted patch entry point used by ForgeCore execution paths.
pub(crate) fn patch_file_authorized(
    action: &AgentAction,
    workspace: &Workspace,
    mode: PatchMode,
    authority: &ExecutionAuthority,
    grant: &AuthorizationGrant,
) -> Result<ExecutionResult, ExecutionError> {
    let started_at = Utc::now();

    enforce_policy(action, grant)?;
    if !has_required_execution_authority(action, authority) {
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
    let patch_value = action
        .payload
        .get(PATCH_FIELD)
        .ok_or(ExecutionError::MissingPayloadField(PATCH_FIELD))?;
    let patch_text = patch_value
        .as_str()
        .ok_or(ExecutionError::PayloadFieldNotString(PATCH_FIELD))?;
    let raw_path = Path::new(path_str);

    let resolved = match workspace.resolve_path(raw_path) {
        PathResolution::Allowed(p) => p,
        PathResolution::Denied(p) => {
            if !raw_path.is_absolute() {
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

    let before_bytes = std::fs::read(&resolved).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ExecutionError::FileNotFound(resolved.clone())
        } else {
            ExecutionError::Io(resolved.clone(), e)
        }
    })?;
    let before_sha256 = sha256_hex(&before_bytes);
    let before_lines: Vec<String> = String::from_utf8_lossy(&before_bytes)
        .lines()
        .map(|line| line.to_string())
        .collect();

    let patch = Patch::parse(patch_text)
        .map_err(|error| ExecutionError::InvalidPatch(error.to_string()))?;
    let apply = patch.apply(&before_lines, crate::patch::ApplyMode::Apply);
    if !apply.failures.is_empty() {
        return Err(ExecutionError::PatchConflict(patch_failures_to_string(
            &apply,
        )));
    }

    let after_text = apply.new_lines.join("\n") + "\n";
    let after_sha256 = sha256_hex(after_text.as_bytes());

    if mode == PatchMode::DryRun {
        return Ok(build_result(
            action,
            ExecutionStatus::Accepted,
            &resolved,
            before_sha256,
            after_sha256,
            patch_text,
            apply.applied_hunks,
            started_at,
        ));
    }

    atomic_write(&resolved, after_text.as_bytes())?;
    Ok(build_result(
        action,
        ExecutionStatus::Succeeded,
        &resolved,
        before_sha256,
        after_sha256,
        patch_text,
        apply.applied_hunks,
        started_at,
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_result(
    action: &AgentAction,
    status: ExecutionStatus,
    path: &Path,
    before_sha256: String,
    after_sha256: String,
    patch_text: &str,
    applied_hunks: usize,
    started_at: chrono::DateTime<Utc>,
) -> ExecutionResult {
    let verification = serde_json::json!({
        "path": path.display().to_string(),
        "before_sha256": before_sha256,
        "after_sha256": after_sha256,
        "patch": patch_text,
        "applied_hunks": applied_hunks,
    });
    ExecutionResult {
        action_id: action.id.clone(),
        status,
        started_at,
        completed_at: Utc::now(),
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        artifacts: vec![path.display().to_string()],
        verification: Some(verification),
        error: None,
    }
}

fn patch_failures_to_string(result: &PatchResult) -> String {
    let mut out = String::new();
    for failure in &result.failures {
        out.push_str(&format!("hunk@{:?}: ", failure.hunk.old_start));
        out.push_str(match &failure.reason {
            crate::patch::PatchFailureReason::StaleContext { .. } => "stale context",
            crate::patch::PatchFailureReason::RangeOutOfBounds { .. } => "range out of bounds",
            crate::patch::PatchFailureReason::Conflict { .. } => "conflict",
            crate::patch::PatchFailureReason::MalformedCounts { .. } => "malformed counts",
        });
        out.push_str("; ");
    }
    out.trim_end_matches("; ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ActionType, AgentAction, Capability, RiskLevel};
    use serde_json::json;

    fn base_action(path: &str, patch: &str) -> AgentAction {
        AgentAction {
            id: "a1".to_string(),
            task_id: "t1".to_string(),
            agent_id: "g1".to_string(),
            action_type: ActionType::PatchFile,
            reason: "apply patch".to_string(),
            risk: RiskLevel::Low,
            capabilities: vec![Capability::PatchFile],
            payload: json!({ "path": path, "patch": patch }),
            expected: json!({}),
        }
    }

    fn patch_with_authority(
        action: &AgentAction,
        workspace: &Workspace,
        mode: PatchMode,
    ) -> Result<ExecutionResult, ExecutionError> {
        let authority = ExecutionAuthority::from_trusted_capabilities([Capability::PatchFile]);
        patch_file_authorized(
            action,
            workspace,
            mode,
            &authority,
            &AuthorizationGrant::none(),
        )
    }

    #[test]
    fn public_patch_fails_closed_without_kernel_authority() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"one\ntwo\n").unwrap();
        let patch_text = "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n one\n-two\n+2nd\n";
        let err = patch_file(&base_action("a.txt", patch_text), &ws, PatchMode::Apply).unwrap_err();
        assert!(matches!(err, ExecutionError::CapabilityDenied));
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "one\ntwo\n"
        );
    }

    #[test]
    fn patch_applies_to_existing_file_with_kernel_authority() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"one\ntwo\nthree\n").unwrap();
        let patch_text = "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n one\n-two\n+2nd\n";
        let result =
            patch_with_authority(&base_action("a.txt", patch_text), &ws, PatchMode::Apply).unwrap();
        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "one\n2nd\nthree\n"
        );
        let verification = result.verification.unwrap();
        assert_eq!(verification["applied_hunks"], 1);
    }

    #[test]
    fn patch_dry_run_does_not_modify() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"one\nold\n").unwrap();
        let patch_text = "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n one\n-old\n+new\n";
        let result =
            patch_with_authority(&base_action("a.txt", patch_text), &ws, PatchMode::DryRun)
                .unwrap();
        assert_eq!(result.status, ExecutionStatus::Accepted);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "one\nold\n"
        );
    }

    #[test]
    fn stale_context_patch_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"uno\nold\n").unwrap();
        let patch_text = "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n one\n-old\n+new\n";
        let err = patch_with_authority(&base_action("a.txt", patch_text), &ws, PatchMode::Apply)
            .unwrap_err();
        assert!(matches!(err, ExecutionError::PatchConflict(_)));
    }

    #[test]
    fn missing_file_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let patch_text = "--- a/a.txt\n+++ b/a.txt\n@@ -1,1 +1,1 @@\n x\n";
        let err = patch_with_authority(
            &base_action("missing.txt", patch_text),
            &ws,
            PatchMode::Apply,
        )
        .unwrap_err();
        assert!(matches!(err, ExecutionError::FileNotFound(_)));
    }

    #[test]
    fn denied_authority_is_rejected_even_when_requested() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"x\n").unwrap();
        let action = base_action("a.txt", "--- a/a\n+++ b/a\n@@ -1,1 +1,1 @@\n-x\n");
        let err = patch_file(&action, &ws, PatchMode::Apply).unwrap_err();
        assert!(matches!(err, ExecutionError::CapabilityDenied));
    }

    #[test]
    fn traversal_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let err = patch_with_authority(
            &base_action("../escape.txt", "--- a\n+++ b\n@@ -1,1 +1,1 @@\n x\n"),
            &ws,
            PatchMode::Apply,
        )
        .unwrap_err();
        assert!(matches!(err, ExecutionError::PathTraversal(_)));
    }

    #[test]
    fn malformed_patch_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"x\n").unwrap();
        let err = patch_with_authority(
            &base_action("a.txt", "not a patch at all"),
            &ws,
            PatchMode::Apply,
        )
        .unwrap_err();
        assert!(matches!(err, ExecutionError::InvalidPatch(_)));
    }

    #[test]
    fn high_risk_patch_requires_trusted_grant_and_authority() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"one\nold\n").unwrap();
        let patch_text = "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n one\n-old\n+new\n";
        let mut action = base_action("a.txt", patch_text);
        action.risk = RiskLevel::High;
        let authority = ExecutionAuthority::from_trusted_capabilities([Capability::PatchFile]);
        assert!(matches!(
            patch_file_authorized(
                &action,
                &ws,
                PatchMode::Apply,
                &authority,
                &AuthorizationGrant::none(),
            )
            .unwrap_err(),
            ExecutionError::RequiresApproval
        ));
        let result = patch_file_authorized(
            &action,
            &ws,
            PatchMode::Apply,
            &authority,
            &AuthorizationGrant::approved("approval-1"),
        )
        .unwrap();
        assert_eq!(result.status, ExecutionStatus::Succeeded);
    }
}
