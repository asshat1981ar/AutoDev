//! The `patch_file` executor — apply a unified-diff patch to a file.
//!
//! This is a small, safe capability: it reuses the existing patch engine
//! (`Patch::apply`) and the atomic-write path from `write_file`, so it spawns
//! no process and adds no new security boundary beyond the tier-1 workspace
//! confinement and capability gate already in place.
//!
//! Pipeline:
//!
//! ```text
//! AgentAction
//!   → validate_action        (structural invariants)
//!   → capability check       (is the `patch_file` capability granted?)
//!   → evaluate_policy        (risk → Allow / RequireApproval / Deny)
//!   → workspace.resolve_path (containment + symlink/traversal defense)
//!   → read target            (existing content for before-hash)
//!   → parse patch            (payload.patch = unified diff)
//!   → apply patch            (deterministic, context-validated)
//!   → atomic write           (only on clean apply)
//!   → evidence               (before/after hashes + applied patch)
//! ```

use std::path::Path;

use chrono::Utc;

use crate::action::AgentAction;
use crate::error::ExecutionError;
use crate::evidence::{sha256_hex, ExecutionResult, ExecutionStatus};
use crate::patch::{Patch, PatchResult};
use crate::policy::{evaluate_policy, has_required_capability, PolicyDecision};
use crate::workspace::{PathResolution, Workspace};
use crate::write::atomic_write;

/// The payload fields `patch_file` understands.
const PATH_FIELD: &str = "path";
const PATCH_FIELD: &str = "patch";

/// Whether a patch should write the result or only validate (dry-run).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchMode {
    /// Validate the patch and compute the result without writing.
    DryRun,
    /// Apply the patch and write the result.
    Apply,
}

/// Execute a `patch_file` action.
///
/// Returns a schema-conformant [`ExecutionResult`]. Policy, workspace, patch,
/// or I/O failures are surfaced as structured [`ExecutionError`]s. In
/// [`PatchMode::DryRun`] the filesystem is never modified.
pub fn patch_file(
    action: &AgentAction,
    workspace: &Workspace,
    mode: PatchMode,
) -> Result<ExecutionResult, ExecutionError> {
    let started_at = Utc::now();

    // 1. Structural validation.
    evaluate_policy(action)?;

    // 2. Capability check: patching requires the `patch_file` capability.
    if !has_required_capability(action) {
        return Err(ExecutionError::CapabilityDenied);
    }

    // 3. Risk-based policy.
    match evaluate_policy(action)? {
        PolicyDecision::Allow => {}
        PolicyDecision::RequireApproval => return Err(ExecutionError::RequiresApproval),
        PolicyDecision::Deny => return Err(ExecutionError::CapabilityDenied),
    }

    // 4. Extract and validate the payload.
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

    // 5. Resolve against the workspace (containment + symlink/traversal).
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

    // 6. Read the existing file (for before-hash + patch source).
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
        .map(|s| s.to_string())
        .collect();

    // 7. Parse the patch.
    let patch =
        Patch::parse(patch_text).map_err(|e| ExecutionError::InvalidPatch(e.to_string()))?;

    // 8. Apply the patch (deterministic).
    let apply = patch.apply(&before_lines, crate::patch::ApplyMode::Apply);
    if !apply.failures.is_empty() {
        return Err(ExecutionError::PatchConflict(patch_failures_to_string(
            &apply,
        )));
    }

    let after_text = apply.new_lines.join("\n") + "\n";
    let after_sha256 = sha256_hex(after_text.as_bytes());

    // 9. Dry-run stops here; the filesystem is untouched.
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

    // 10. Atomic write the result.
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

/// Build a schema-conformant result for a patch.
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

/// Summarize patch failures into a stable error message.
fn patch_failures_to_string(result: &PatchResult) -> String {
    let mut out = String::new();
    for f in &result.failures {
        out.push_str(&format!("hunk@{:?}: ", f.hunk.old_start));
        out.push_str(match &f.reason {
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

    #[test]
    fn patch_applies_to_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"one\ntwo\nthree\n").unwrap();
        let patch_text = "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n one\n-two\n+2nd\n";

        let result = patch_file(&base_action("a.txt", patch_text), &ws, PatchMode::Apply).unwrap();
        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "one\n2nd\nthree\n"
        );
        let v = result.verification.unwrap();
        assert!(v["applied_hunks"] == 1);
        assert!(v["before_sha256"].is_string());
        assert!(v["after_sha256"].is_string());
    }

    #[test]
    fn patch_dry_run_does_not_modify() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"one\nold\n").unwrap();
        let patch_text = "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n one\n-old\n+new\n";

        let result = patch_file(&base_action("a.txt", patch_text), &ws, PatchMode::DryRun).unwrap();
        assert_eq!(result.status, ExecutionStatus::Accepted);
        // File unchanged.
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "one\nold\n"
        );
    }

    #[test]
    fn stale_context_patch_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        // Context line "one" changed to "uno".
        std::fs::write(dir.path().join("a.txt"), b"uno\nold\n").unwrap();
        let patch_text = "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n one\n-old\n+new\n";
        let err = patch_file(&base_action("a.txt", patch_text), &ws, PatchMode::Apply).unwrap_err();
        assert!(matches!(err, ExecutionError::PatchConflict(_)));
    }

    #[test]
    fn missing_file_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let patch_text = "--- a/a.txt\n+++ b/a.txt\n@@ -1,1 +1,1 @@\n x\n";
        let err = patch_file(
            &base_action("missing.txt", patch_text),
            &ws,
            PatchMode::Apply,
        )
        .unwrap_err();
        assert!(matches!(err, ExecutionError::FileNotFound(_)));
    }

    #[test]
    fn denied_capability_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        std::fs::write(dir.path().join("a.txt"), b"x\n").unwrap();
        let mut a = base_action("a.txt", "--- a/a\n+++ b/a\n@@ -1,1 +1,1 @@\n-x\n");
        a.capabilities = vec![]; // no patch_file capability
        let err = patch_file(&a, &ws, PatchMode::Apply).unwrap_err();
        assert!(matches!(err, ExecutionError::CapabilityDenied));
    }

    #[test]
    fn traversal_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 4096).unwrap();
        let err = patch_file(
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
        let err = patch_file(
            &base_action("a.txt", "not a patch at all"),
            &ws,
            PatchMode::Apply,
        )
        .unwrap_err();
        assert!(matches!(err, ExecutionError::InvalidPatch(_)));
    }
}
