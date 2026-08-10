//! ForgeCore is the trusted execution boundary for AutoDev.
//!
//! Agents produce intent. ForgeCore executes only intent that has passed
//! policy evaluation and workspace confinement. The initial implementation
//! provides the typed agent-action protocol and a single real, read-only
//! operation (`read_file`). It deliberately contains no privileged filesystem
//! mutation or process execution.

pub mod action;
pub mod error;
pub mod evidence;
pub mod policy;
pub mod read;
pub mod workspace;

pub use action::{ActionType, AgentAction, Capability, RiskLevel};
pub use error::{ExecutionError, ExecutionErrorKind};
pub use evidence::{ExecutionResult, ExecutionStatus, ReadMetadata};
pub use policy::{evaluate_policy, has_required_capability, validate_action, PolicyDecision};
pub use read::read_file;
pub use workspace::{PathResolution, Workspace};

use chrono::Utc;

/// A validated, authorized action ready for execution, bound to a workspace.
#[derive(Debug, Clone)]
pub struct ExecutableAction {
    pub action: AgentAction,
    pub workspace: Workspace,
}

impl ExecutableAction {
    /// Construct an executable action. The workspace root is canonicalized
    /// eagerly.
    pub fn new(action: AgentAction, workspace: Workspace) -> Self {
        ExecutableAction { action, workspace }
    }
}

/// The top-level execution entry point.
///
/// This is the only path that can produce filesystem side effects (currently
/// limited to read-only access). It dispatches on the action type and returns
/// schema-conformant evidence.
pub fn execute(exec: &ExecutableAction) -> Result<ExecutionResult, ExecutionError> {
    let mut result = match exec.action.action_type {
        ActionType::ReadFile => read::read_file(&exec.action, &exec.workspace)?,
        other => {
            return Err(ExecutionError::UnsupportedAction(
                other.as_str().to_string(),
            ))
        }
    };
    result.action_id = exec.action.id.clone();
    Ok(result)
}

/// Dry-run preview: evaluates policy without touching the filesystem.
///
/// Returns what *would* happen, without performing any privileged effect.
pub fn dry_run(action: &AgentAction) -> Result<ExecutionResult, ExecutionError> {
    evaluate_policy(action)?;
    if !has_required_capability(action) {
        return Err(ExecutionError::CapabilityDenied);
    }
    let now = Utc::now();
    Ok(ExecutionResult {
        action_id: action.id.clone(),
        status: ExecutionStatus::Accepted,
        started_at: now,
        completed_at: now,
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        artifacts: vec![],
        verification: None,
        error: None,
    })
}
