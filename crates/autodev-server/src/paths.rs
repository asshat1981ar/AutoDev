use std::path::{Path, PathBuf};

use forge_core::Workspace;
use thiserror::Error;

/// Choose a local state directory that is a sibling of the execution workspace.
///
/// Keeping trusted orchestration state outside the workspace makes it
/// unreachable through ForgeCore's confined write/patch execution adapters.
pub fn default_state_dir(workspace: &Workspace) -> PathBuf {
    let workspace_root = workspace.root();
    let name = workspace_root
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("workspace");
    match workspace_root.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            parent.join(format!(".autodev-state-{name}"))
        }
        _ => std::env::temp_dir().join(format!("autodev-state-{name}")),
    }
}

/// Create and canonicalize the control-plane state directory, then prove it is
/// not writable through the configured execution workspace.
pub fn validate_control_plane_paths(
    workspace: &Workspace,
    state_dir: impl AsRef<Path>,
) -> Result<PathBuf, ControlPlanePathError> {
    std::fs::create_dir_all(state_dir.as_ref())?;
    let canonical_state = std::fs::canonicalize(state_dir.as_ref())?;
    let workspace_root = workspace.root();

    if canonical_state == workspace_root || canonical_state.starts_with(workspace_root) {
        return Err(ControlPlanePathError::StateInsideWorkspace {
            state_dir: canonical_state,
            workspace: workspace_root.to_path_buf(),
        });
    }

    Ok(canonical_state)
}

#[derive(Debug, Error)]
pub enum ControlPlanePathError {
    #[error(
        "control-plane state directory '{}' must be outside execution workspace '{}'",
        state_dir.display(),
        workspace.display()
    )]
    StateInsideWorkspace {
        state_dir: PathBuf,
        workspace: PathBuf,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
