//! Structured error types for the ForgeCore execution kernel.
//!
//! Errors are deliberately specific and stable so callers can match on them
//! and surface precise feedback (e.g. "file too large") without parsing prose.

use std::path::PathBuf;

use thiserror::Error;

/// Errors produced while validating or executing an agent action.
#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("action id must not be empty")]
    MissingActionId,
    #[error("task id must not be empty")]
    MissingTaskId,
    #[error("agent id must not be empty")]
    MissingAgentId,
    #[error("action reason must not be empty")]
    MissingReason,
    #[error("critical actions require the 'approval:critical' capability")]
    CriticalApprovalRequired,

    #[error("action type '{0}' is not supported by this executor")]
    UnsupportedAction(String),
    #[error("action was denied by capability policy")]
    CapabilityDenied,
    #[error("action requires approval before execution")]
    RequiresApproval,

    #[error("payload must be a JSON object")]
    PayloadNotObject,
    #[error("payload field '{0}' is missing")]
    MissingPayloadField(&'static str),
    #[error("payload field '{0}' must be a string")]
    PayloadFieldNotString(&'static str),

    #[error("path '{0}' is not contained within the workspace")]
    PathOutsideWorkspace(PathBuf),
    #[error("path traversal detected in '{0}'")]
    PathTraversal(PathBuf),
    #[error("symlink escape detected in '{0}'")]
    SymlinkEscape(PathBuf),
    #[error("path '{0}' is invalid")]
    InvalidPath(PathBuf),

    #[error("file not found: '{0}'")]
    FileNotFound(PathBuf),
    #[error("path '{0}' is a directory, expected a file")]
    IsDirectory(PathBuf),
    #[error("patch is malformed: {0}")]
    InvalidPatch(String),
    #[error("patch could not be applied: {0}")]
    PatchConflict(String),
    #[error("file '{0}' exceeds the maximum allowed size of {1} bytes")]
    OversizedFile(PathBuf, u64),

    #[error("file '{0}' is not valid UTF-8")]
    InvalidUtf8(PathBuf),

    #[error("'{0}' is not a git repository")]
    NotARepository(PathBuf),
    #[error("git operation '{0}' failed: {1}")]
    GitFailed(&'static str, String),
    #[error("git operation requires the '{0}' capability")]
    GitCapabilityDenied(&'static str),
    #[error("git operation '{0}' requires approval")]
    GitRequiresApproval(&'static str),
    #[error("git operation '{0}' is forbidden without explicit policy authorization")]
    GitOperationForbidden(String),

    #[error("process execution requires an enabled process sandbox (tier-2, fail-closed)")]
    ProcessSandboxRequired,
    #[error("command execution timed out after {0} seconds")]
    ProcessTimeout(u64),
    #[error("command output exceeded the {0} byte limit")]
    ProcessOutputTooLarge(u64),
    #[error("command contains shell metacharacters and was refused: {0}")]
    UnsafeCommand(String),

    #[error("I/O error accessing '{0}': {1}")]
    Io(PathBuf, std::io::Error),
}

impl ExecutionError {
    /// The requirement (capability, path, approval) that failed, for stable
    /// machine matching.
    pub fn kind(&self) -> ExecutionErrorKind {
        match self {
            ExecutionError::MissingActionId
            | ExecutionError::MissingTaskId
            | ExecutionError::MissingAgentId
            | ExecutionError::MissingReason
            | ExecutionError::CriticalApprovalRequired => ExecutionErrorKind::Validation,
            ExecutionError::UnsupportedAction(_) => ExecutionErrorKind::Unsupported,
            ExecutionError::CapabilityDenied | ExecutionError::RequiresApproval => {
                ExecutionErrorKind::Policy
            }
            ExecutionError::PayloadNotObject
            | ExecutionError::MissingPayloadField(_)
            | ExecutionError::PayloadFieldNotString(_) => ExecutionErrorKind::InvalidPayload,
            ExecutionError::PathOutsideWorkspace(_)
            | ExecutionError::PathTraversal(_)
            | ExecutionError::SymlinkEscape(_)
            | ExecutionError::InvalidPath(_) => ExecutionErrorKind::Workspace,
            ExecutionError::FileNotFound(_)
            | ExecutionError::IsDirectory(_)
            | ExecutionError::OversizedFile(_, _)
            | ExecutionError::InvalidUtf8(_) => ExecutionErrorKind::Read,
            ExecutionError::InvalidPatch(_) | ExecutionError::PatchConflict(_) => {
                ExecutionErrorKind::Patch
            }
            ExecutionError::NotARepository(_)
            | ExecutionError::GitFailed(_, _)
            | ExecutionError::GitCapabilityDenied(_)
            | ExecutionError::GitRequiresApproval(_)
            | ExecutionError::GitOperationForbidden(_) => ExecutionErrorKind::Git,
            ExecutionError::ProcessSandboxRequired
            | ExecutionError::ProcessTimeout(_)
            | ExecutionError::ProcessOutputTooLarge(_)
            | ExecutionError::UnsafeCommand(_) => ExecutionErrorKind::Process,
            ExecutionError::Io(_, _) => ExecutionErrorKind::Io,
        }
    }
}

/// A coarse, stable classification of an error for programmatic handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionErrorKind {
    Validation,
    Policy,
    Workspace,
    InvalidPayload,
    Read,
    Git,
    Patch,
    Unsupported,
    Process,
    Io,
}
