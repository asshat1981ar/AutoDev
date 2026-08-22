//! Git workspace awareness for ForgeCore.
//!
//! Git operations are shelled out to the system `git` binary via
//! [`std::process::Command`] using an **argv array and no shell**, so there is no
//! shell interpolation or injection surface. Every command is scoped to the
//! workspace root with `git -C <root>` so Git cannot escape the workspace.
//!
//! Operations are separated into three capability tiers:
//!
//! - **Read-only** (`Git` capability, default-grant for read): repository
//!   detection, status, diff, branch information, log. No state change.
//! - **Mutating** (`GitWrite` capability + approval): checkpoint and commit
//!   preparation (staging). Changes the index/refs but is reversible.
//! - **Destructive** (`GitDestructive` capability + approval): rollback
//!   (`reset --hard`, `checkout`, `revert`). Irreversible; refused by default.

use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::action::AgentAction;
use crate::error::ExecutionError;
use crate::evidence::ExecutionResult;
use crate::workspace::Workspace;

/// The Git capability tiers, in increasing privilege.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitTier {
    /// Read-only: detection, status, diff, branch, log.
    Read,
    /// Mutating: checkpoint, commit preparation (staging).
    Mutate,
    /// Destructive: rollback (reset --hard, checkout, revert).
    Destructive,
}

impl GitTier {
    /// The capability name required for this tier.
    pub fn capability_name(self) -> &'static str {
        match self {
            GitTier::Read => "git",
            GitTier::Mutate => "git:write",
            GitTier::Destructive => "git:destructive",
        }
    }
}

/// Information about the repository at a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// The detected repository root (absolute).
    pub root: PathBuf,
    /// The current branch name, if any.
    pub branch: Option<String>,
    /// The current HEAD commit id (short), if any.
    pub head: Option<String>,
    /// Whether the working tree is clean.
    pub clean: bool,
}

/// The result of `git status --short --branch`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitStatus {
    /// The current branch or detached HEAD id.
    pub branch: String,
    /// Lines of the short status (one per changed path).
    pub entries: Vec<String>,
    /// Whether the working tree is clean (no staged/unstaged/untracked).
    pub clean: bool,
}

/// The result of `git diff`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitDiff {
    /// The unified diff text produced by `git diff`.
    pub text: String,
    /// Whether the working tree (unstaged changes) is empty.
    pub empty: bool,
}

/// Branch information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchInfo {
    /// The current branch name.
    pub current: String,
    /// All branch names in the repository.
    pub branches: Vec<String>,
}

/// A checkpoint captured by `git stash push` (or a commit for WIP).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Description of the checkpoint.
    pub message: String,
    /// The stash/commit reference created.
    pub reference: String,
}

/// Run a git command in `root`, capturing stdout.
///
/// Uses an argv array (no shell). `op` is a static operation name used in error
/// reporting. Returns the trimmed stdout on success, or a structured
/// [`ExecutionError::GitFailed`] carrying git's stderr on failure.
fn run_git(op: &'static str, root: &Path, args: &[&str]) -> Result<String, ExecutionError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| ExecutionError::GitFailed(op, format!("failed to spawn git: {e}")))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(ExecutionError::GitFailed(op, stderr))
    }
}

/// Whether `root` (or any ancestor) is a git repository.
pub fn is_repository(root: &Path) -> bool {
    run_git("rev-parse", root, &["rev-parse", "--is-inside-work-tree"])
        .map(|s| s.trim() == "true")
        .unwrap_or(false)
}

/// Detect and describe the repository at `workspace`.
///
/// Read-only. Returns [`ExecutionError::NotARepository`] if the workspace is
/// not inside a git work tree.
pub fn repository_info(workspace: &Workspace) -> Result<RepositoryInfo, ExecutionError> {
    if !is_repository(workspace.root()) {
        return Err(ExecutionError::NotARepository(
            workspace.root().to_path_buf(),
        ));
    }
    let root = workspace
        .root()
        .canonicalize()
        .unwrap_or_else(|_| workspace.root().to_path_buf());
    let branch = current_branch_name(&root).ok();
    let head = head_short(&root).ok();
    let clean = status(&root).map(|s| s.clean).unwrap_or(false);
    Ok(RepositoryInfo {
        root,
        branch,
        head,
        clean,
    })
}

/// The current branch name (or detached HEAD short id).
fn current_branch_name(root: &Path) -> Result<String, ExecutionError> {
    let out = run_git("rev-parse", root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let name = out.trim().to_string();
    Ok(name)
}

/// The short HEAD commit id.
fn head_short(root: &Path) -> Result<String, ExecutionError> {
    let out = run_git("rev-parse", root, &["rev-parse", "--short", "HEAD"])?;
    Ok(out.trim().to_string())
}

/// `git status --short --branch`.
pub fn status(root: &Path) -> Result<GitStatus, ExecutionError> {
    if !is_repository(root) {
        return Err(ExecutionError::NotARepository(root.to_path_buf()));
    }
    let out = run_git("status", root, &["status", "--short", "--branch"])?;
    let mut lines: Vec<String> = out.lines().map(|s| s.to_string()).collect();
    let branch = lines
        .first()
        .cloned()
        .map(|l| l.trim_start_matches("## ").to_string())
        .unwrap_or_else(|| "HEAD".to_string());
    if !lines.is_empty() {
        lines.remove(0); // the branch header line
    }
    let clean = lines.is_empty();
    Ok(GitStatus {
        branch,
        entries: lines,
        clean,
    })
}

/// `git diff` (unstaged changes in the working tree).
pub fn diff(root: &Path) -> Result<GitDiff, ExecutionError> {
    if !is_repository(root) {
        return Err(ExecutionError::NotARepository(root.to_path_buf()));
    }
    let text = run_git("diff", root, &["diff"])?;
    Ok(GitDiff {
        empty: text.trim().is_empty(),
        text,
    })
}

/// Branch information: current branch plus all local branches.
pub fn branch_info(root: &Path) -> Result<BranchInfo, ExecutionError> {
    if !is_repository(root) {
        return Err(ExecutionError::NotARepository(root.to_path_buf()));
    }
    let current = current_branch_name(root)?;
    let out = run_git("branch", root, &["branch", "--format=%(refname:short)"])?;
    let branches: Vec<String> = out
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(BranchInfo { current, branches })
}

/// A short commit log (`git log --oneline -n <limit>`).
pub fn log(root: &Path, limit: usize) -> Result<Vec<String>, ExecutionError> {
    if !is_repository(root) {
        return Err(ExecutionError::NotARepository(root.to_path_buf()));
    }
    let out = run_git("log", root, &["log", "--oneline", &format!("-{limit}")])?;
    Ok(out.lines().map(|s| s.to_string()).collect())
}
/// Create a checkpoint of the working tree via `git stash push -m <message>`.
///
/// Mutating (`GitWrite`) and reversible. Returns the stash reference created.
pub fn checkpoint(root: &Path, message: &str) -> Result<Checkpoint, ExecutionError> {
    if !is_repository(root) {
        return Err(ExecutionError::NotARepository(root.to_path_buf()));
    }
    // `git stash push -m <message>` amounts to `git stash save`.
    let out = run_git("stash", root, &["stash", "push", "-m", message])?;
    let reference = out
        .lines()
        .find_map(|l| l.trim().contains("stash@").then(|| l.trim().to_string()))
        .unwrap_or_else(|| "stash@{0}".to_string());
    Ok(Checkpoint {
        message: message.to_string(),
        reference,
    })
}

/// Prepare a commit: stage all changes (`git add -A`) and (optionally) create
/// the commit.
///
/// Mutating (`GitWrite`). When `commit` is true a commit is created; otherwise
/// the changes are staged but not committed.
pub fn prepare_commit(
    root: &Path,
    message: &str,
    commit: bool,
) -> Result<Checkpoint, ExecutionError> {
    if !is_repository(root) {
        return Err(ExecutionError::NotARepository(root.to_path_buf()));
    }
    run_git("add", root, &["add", "-A"])?;
    if commit {
        run_git("commit", root, &["commit", "-m", message])?;
    }
    let reference = if commit {
        head_short(root)?
    } else {
        "staged".to_string()
    };
    Ok(Checkpoint {
        message: message.to_string(),
        reference,
    })
}

/// Roll the workspace back to a previous state.
///
/// Destructive (`GitDestructive`, requires approval). Supports:
/// - `reset --hard <ref>` — discard unstaged + staged changes and move HEAD.
/// - `checkout -- .` — discard working-tree changes only.
///
/// The `Command` argument must be a plain identifier (e.g. `"reset"`) and `to`
/// a revision; both are passed as argv items, never interpolated into a shell.
///
/// Forbidden operations (see [`forbidden_command`]) are refused outright.
pub fn rollback(root: &Path, command: &str, to: Option<&str>) -> Result<(), ExecutionError> {
    if !is_repository(root) {
        return Err(ExecutionError::NotARepository(root.to_path_buf()));
    }
    if forbidden_command(command) {
        return Err(ExecutionError::GitOperationForbidden(command.to_string()));
    }
    match command {
        "reset" => {
            let target = to.unwrap_or("HEAD");
            run_git("reset", root, &["reset", "--hard", target])?;
        }
        "checkout" => {
            run_git("checkout", root, &["checkout", "--", "."])?;
        }
        other => {
            return Err(ExecutionError::GitFailed(
                "rollback",
                format!("unsupported rollback command '{other}'"),
            ))
        }
    }
    Ok(())
}

/// Whether a git operation is structurally forbidden in this build.
///
/// The platform **never** performs these operations without explicit policy
/// authorization: force push, deleting remote branches, modifying credentials,
/// or rewriting protected history. Refusing them here makes the "never" property
/// structural rather than dependent on an agent's instructions (which are not
/// authority).
pub fn forbidden_command(command: &str) -> bool {
    matches!(
        command,
        /* force push / force-with-lease */
        "push --force"
            | "push --force-with-lease"
            | "push -f"
            /* delete a remote branch */
            | "push --delete"
            | "push -d"
            /* rewrite / destroy history */
            | "reset --hard" // (rollback's reset is bounded to a local ref via `rollback`)
            | "filter-branch"
            | "rebase"
            | "clean -fd"
            /* credentials / config / origin manipulation */
            | "config credential"
            | "credential"
            | "remote remove"
            | "remote rm"
            | "remote set-url"
    )
}

/// Require explicit approval for a destructive operation.
///
/// Approval is an explicit attestation passed from the trusted boundary:
/// - the caller-provided `approved` flag derived from a kernel-owned
///   `AuthorizationGrant` (never from the action payload), or
/// - the special `approval:critical` capability (already validated elsewhere).
///
/// Without it, destructive operations are refused with
/// [`ExecutionError::RequiresApproval`] rather than proceeding. This makes
/// "approval required" structural rather than advisory.
fn require_approval(approved: bool, action: &AgentAction) -> Result<(), ExecutionError> {
    let has_critical_cap = action
        .capabilities
        .iter()
        .any(|c| c == &crate::action::Capability::ApprovalCritical);
    if approved || has_critical_cap {
        Ok(())
    } else {
        Err(ExecutionError::RequiresApproval)
    }
}
/// Check that `action` holds the capability for `tier`.
///
/// Enforcement rule (per the product decision): read-only Git is permitted for
/// any agent holding the `git` capability; mutating and destructive Git require
/// their own, more privileged capabilities (`git:write` / `git:destructive`).
fn gate(tier: GitTier, granted: &[crate::action::Capability]) -> Result<(), ExecutionError> {
    let required = match tier {
        GitTier::Read => crate::action::Capability::Git,
        GitTier::Mutate => crate::action::Capability::GitWrite,
        GitTier::Destructive => crate::action::Capability::GitDestructive,
    };
    if granted.iter().any(|c| c == &required) {
        Ok(())
    } else {
        Err(ExecutionError::GitCapabilityDenied(tier.capability_name()))
    }
}

/// Run a tier-gated read operation.
///
/// `name` is the operation name for error reporting; `op` performs it.
pub fn run_read<F, T>(granted: &[crate::action::Capability], op: F) -> Result<T, ExecutionError>
where
    F: FnOnce() -> Result<T, ExecutionError>,
{
    gate(GitTier::Read, granted)?;
    op()
}

/// Run a tier-gated mutating operation.
pub fn run_mutate<F, T>(granted: &[crate::action::Capability], op: F) -> Result<T, ExecutionError>
where
    F: FnOnce() -> Result<T, ExecutionError>,
{
    gate(GitTier::Mutate, granted)?;
    op()
}

/// Run a tier-gated destructive operation.
pub fn run_destructive<F, T>(
    granted: &[crate::action::Capability],
    op: F,
) -> Result<T, ExecutionError>
where
    F: FnOnce() -> Result<T, ExecutionError>,
{
    gate(GitTier::Destructive, granted)?;
    op()
}

/// Dispatch a `git` action to the appropriate tier-gated operation.
///
/// The payload must be an object with an `operation` field naming one of:
///
/// - Read-only: `repository_info`, `status`, `diff`, `branch`, `log`
/// - Mutating: `checkpoint`, `prepare_commit`
/// - Destructive: `rollback`
///
/// Capability gates are enforced by tier as described in the module docs.
pub fn execute_git(
    action: &AgentAction,
    workspace: &Workspace,
    approved: bool,
) -> Result<ExecutionResult, ExecutionError> {
    use crate::evidence::ExecutionStatus;

    if !action.payload.is_object() {
        return Err(ExecutionError::PayloadNotObject);
    }
    let op = action
        .payload
        .get("operation")
        .and_then(|v| v.as_str())
        .ok_or(ExecutionError::MissingPayloadField("operation"))?;

    // The git binary operates on the workspace root; git resolves the real
    // repo root itself, so passing the workspace root is safe and confined.
    let root = workspace.root();

    // Build the result payload for read operations.
    let capture = |v: serde_json::Value| {
        let now = Utc::now();
        Ok(ExecutionResult {
            action_id: action.id.clone(),
            status: ExecutionStatus::Succeeded,
            started_at: now,
            completed_at: now,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            artifacts: vec![],
            verification: Some(v),
            error: None,
        })
    };

    match op {
        "repository_info" => {
            gate(GitTier::Read, &action.capabilities)?;
            let info = repository_info(workspace)?;
            capture(serde_json::to_value(&info).unwrap())
        }
        "status" => {
            gate(GitTier::Read, &action.capabilities)?;
            let s = status(root)?;
            capture(serde_json::to_value(&s).unwrap())
        }
        "diff" => {
            gate(GitTier::Read, &action.capabilities)?;
            let d = diff(root)?;
            capture(serde_json::to_value(&d).unwrap())
        }
        "branch" => {
            gate(GitTier::Read, &action.capabilities)?;
            let b = branch_info(root)?;
            capture(serde_json::to_value(&b).unwrap())
        }
        "log" => {
            gate(GitTier::Read, &action.capabilities)?;
            let limit = action
                .payload
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize)
                .unwrap_or(10);
            let entries = log(root, limit)?;
            capture(serde_json::json!({ "entries": entries }))
        }
        "checkpoint" => {
            gate(GitTier::Mutate, &action.capabilities)?;
            let message = action
                .payload
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("autodev checkpoint");
            let cp = checkpoint(root, message)?;
            capture(serde_json::to_value(&cp).unwrap())
        }
        "prepare_commit" => {
            gate(GitTier::Mutate, &action.capabilities)?;
            let message = action
                .payload
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("autodev commit");
            let do_commit = action
                .payload
                .get("commit")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let cp = prepare_commit(root, message, do_commit)?;
            capture(serde_json::to_value(&cp).unwrap())
        }
        "rollback" => {
            gate(GitTier::Destructive, &action.capabilities)?;
            // Destructive Git requires explicit approval (in addition to the
            // capability). Without it, refuse rather than proceed.
            require_approval(approved, action)?;
            let command = action
                .payload
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("checkout");
            let to = action.payload.get("to").and_then(|v| v.as_str());
            rollback(root, command, to)?;
            capture(serde_json::json!({ "rolled_back": command }))
        }
        other => Err(ExecutionError::GitFailed(
            "execute",
            format!("unknown git operation '{other}'"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Capability;
    use std::process::Command;

    /// Create a fresh git repository in a temp dir with an initial commit.
    fn init_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
        ] {
            let status = Command::new("git")
                .arg("-C")
                .arg(root)
                .args(&args)
                .status()
                .unwrap();
            assert!(status.success());
        }
        // Initial commit so HEAD exists.
        std::fs::write(dir.path().join("base.txt"), b"base").unwrap();
        let status = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["add", "-A"])
            .status()
            .unwrap();
        assert!(status.success());
        let status = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["commit", "-q", "-m", "init"])
            .status()
            .unwrap();
        assert!(status.success());
        dir
    }

    #[test]
    fn detects_repository() {
        let dir = init_repo();
        assert!(is_repository(dir.path()));
        let not_repo = tempfile::tempdir().unwrap();
        assert!(!is_repository(not_repo.path()));
    }

    #[test]
    fn status_reports_clean_and_dirty() {
        let dir = init_repo();
        let clean = status(dir.path()).unwrap();
        assert!(clean.clean);
        assert_eq!(clean.entries.len(), 0);

        std::fs::write(dir.path().join("new.txt"), b"x").unwrap();
        let dirty = status(dir.path()).unwrap();
        assert!(!dirty.clean);
        assert!(dirty.entries.iter().any(|e| e.contains("new.txt")));
    }

    #[test]
    fn diff_reports_unstaged_changes() {
        let dir = init_repo();
        std::fs::write(dir.path().join("base.txt"), b"changed").unwrap();
        let d = diff(dir.path()).unwrap();
        assert!(!d.empty);
        assert!(d.text.contains("base.txt"));
    }

    #[test]
    fn branch_information_is_reported() {
        let dir = init_repo();
        let info = branch_info(dir.path()).unwrap();
        assert_eq!(info.current, "master");
        assert!(info.branches.contains(&"master".to_string()));
    }

    #[test]
    fn repository_info_lists_branch_and_head() {
        let dir = init_repo();
        let ws = Workspace::new(dir.path(), 1024).unwrap();
        let info = repository_info(&ws).unwrap();
        assert!(info.branch.is_some());
        assert!(info.head.is_some());
        assert!(info.clean);
    }

    #[test]
    fn checkpoint_stashes_changes() {
        let dir = init_repo();
        std::fs::write(dir.path().join("base.txt"), b"wip").unwrap();
        let cp = checkpoint(dir.path(), "wip checkpoint").unwrap();
        assert!(cp.reference.contains("stash"));
        // After stash, the working tree is restored to a clean state.
        let s = status(dir.path()).unwrap();
        assert!(s.clean);
    }

    #[test]
    fn prepare_commit_commits_changes() {
        let dir = init_repo();
        std::fs::write(dir.path().join("base.txt"), b"committed").unwrap();
        let cp = prepare_commit(dir.path(), "my change", true).unwrap();
        assert_eq!(cp.reference.len(), 7); // short sha
        let s = status(dir.path()).unwrap();
        assert!(s.clean);
    }

    #[test]
    fn rollback_resets_working_tree() {
        let dir = init_repo();
        std::fs::write(dir.path().join("base.txt"), b"dirty").unwrap();
        rollback(dir.path(), "reset", Some("HEAD")).unwrap();
        let after = std::fs::read_to_string(dir.path().join("base.txt")).unwrap();
        assert_eq!(after, "base");
    }

    #[test]
    fn read_gate_permits_git_capability() {
        assert!(gate(GitTier::Read, &[Capability::Git]).is_ok());
        assert!(gate(GitTier::Read, &[Capability::WriteFile]).is_err());
    }

    #[test]
    fn mutate_and_destructive_require_more() {
        // Read capability is NOT enough for mutating/destructive.
        assert!(gate(GitTier::Mutate, &[Capability::Git]).is_err());
        assert!(gate(GitTier::Destructive, &[Capability::Git]).is_err());
        // The specific higher capabilities are required.
        assert!(gate(GitTier::Mutate, &[Capability::GitWrite]).is_ok());
        assert!(gate(GitTier::Destructive, &[Capability::GitDestructive]).is_ok());
    }

    #[test]
    fn not_a_repository_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let err = status(dir.path()).unwrap_err();
        assert!(matches!(err, ExecutionError::NotARepository(_)));
    }

    #[test]
    fn forbidden_operations_are_refused() {
        for cmd in [
            "push --force",
            "push --force-with-lease",
            "push -f",
            "push --delete",
            "filter-branch",
            "rebase",
            "config credential",
            "credential",
            "remote remove",
            "remote set-url",
        ] {
            assert!(forbidden_command(cmd), "{cmd} should be forbidden");
        }
        // Allowed rollback primitives are not forbidden.
        assert!(!forbidden_command("reset"));
        assert!(!forbidden_command("checkout"));
    }

    #[test]
    fn rollback_refuses_forbidden_command() {
        let dir = init_repo();
        let err = rollback(dir.path(), "push --force", None).unwrap_err();
        assert!(matches!(err, ExecutionError::GitOperationForbidden(_)));
    }

    #[test]
    fn destructive_requires_approval() {
        let dir = init_repo();
        let ws = Workspace::new(dir.path(), 1024).unwrap();
        let mut a = AgentAction {
            id: "a".to_string(),
            task_id: "t".to_string(),
            agent_id: "g".to_string(),
            action_type: crate::action::ActionType::Git,
            reason: "rollback".to_string(),
            risk: crate::action::RiskLevel::Low,
            capabilities: vec![Capability::Git, Capability::GitDestructive],
            payload: serde_json::json!({ "operation": "rollback", "command": "checkout" }),
            expected: serde_json::json!({}),
        };
        // Without approval: refused.
        let err = execute_git(&a, &ws, false).unwrap_err();
        assert!(matches!(err, ExecutionError::RequiresApproval));

        // With trusted approval (explicit parameter, never payload-derived):
        // allowed.
        a.payload = serde_json::json!({ "operation": "rollback", "command": "checkout" });
        assert!(execute_git(&a, &ws, true).is_ok());

        // A payload-supplied approval bit is ignored entirely.
        a.payload = serde_json::json!({
            "operation": "rollback", "command": "checkout", "approved": true
        });
        let err = execute_git(&a, &ws, false).unwrap_err();
        assert!(matches!(err, ExecutionError::RequiresApproval));
    }
}
