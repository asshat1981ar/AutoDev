use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use forge_core::{VerificationRecipe, VerifierEvidence};
use sha2::{Digest, Sha256};

use crate::{RunnerError, VerifierOverlay};

const MAX_STREAM_BYTES: usize = 64 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(25);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepExecution {
    pub evidence: VerifierEvidence,
    pub elapsed_ms: u64,
}

pub fn apply_verifier_overlays(
    crate_root: &Path,
    workspace: &Path,
    overlays: &[VerifierOverlay],
) -> Result<(), RunnerError> {
    let crate_root = fs::canonicalize(crate_root)?;
    let workspace_root = fs::canonicalize(workspace)?;
    let mut prepared = Vec::with_capacity(overlays.len());

    for overlay in overlays {
        let source = confined_source(&crate_root, &overlay.source_path)?;
        let bytes = fs::read(&source)?;
        let actual = sha256(&bytes);
        if actual != overlay.sha256 {
            return Err(RunnerError::OverlayIntegrity(format!(
                "{} expected {}, got {actual}",
                overlay.source_path, overlay.sha256
            )));
        }
        let destination = confined_destination(&workspace_root, &overlay.destination_path)?;
        prepared.push((destination, bytes));
    }

    for (destination, bytes) in prepared {
        let parent = destination.parent().ok_or_else(|| {
            RunnerError::UnsafeOverlayDestination(destination.display().to_string())
        })?;
        fs::create_dir_all(parent)?;
        let canonical_parent = fs::canonicalize(parent)?;
        if !canonical_parent.starts_with(&workspace_root) {
            return Err(RunnerError::UnsafeOverlayDestination(
                destination.display().to_string(),
            ));
        }
        fs::write(destination, bytes)?;
    }
    Ok(())
}

pub fn run_verifier(
    workspace: &Path,
    recipe: &VerificationRecipe,
) -> Result<Vec<StepExecution>, RunnerError> {
    let workspace_root = fs::canonicalize(workspace)?;
    let mut executions = Vec::with_capacity(recipe.steps.len());

    for step in &recipe.steps {
        let current_dir = confined_working_directory(&workspace_root, &step.working_directory)?;
        let started = Instant::now();
        let mut command = Command::new(&step.program);
        command
            .args(&step.args)
            .current_dir(current_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        command.process_group(0);

        let mut child = command.spawn().map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                RunnerError::MissingExecutable(step.program.clone())
            } else {
                RunnerError::Io(error)
            }
        })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| RunnerError::Io(io::Error::other("verifier stdout pipe missing")))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| RunnerError::Io(io::Error::other("verifier stderr pipe missing")))?;
        let stdout_reader = capture_stream(stdout);
        let stderr_reader = capture_stream(stderr);

        let wait_result = wait_with_timeout(
            &mut child,
            Duration::from_secs(u64::from(step.timeout_seconds)),
        );
        let stdout = join_capture(stdout_reader)?;
        let stderr = join_capture(stderr_reader)?;
        let (status, timed_out) = wait_result.map_err(RunnerError::Io)?;
        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

        executions.push(StepExecution {
            evidence: VerifierEvidence {
                step_id: step.id.clone(),
                required: step.required,
                passed: status.success() && !timed_out,
                exit_code: status.code(),
                stdout_sha256: sha256(&stdout),
                stderr_sha256: sha256(&stderr),
                timed_out,
            },
            elapsed_ms,
        });
    }

    Ok(executions)
}

fn capture_stream<R>(mut reader: R) -> thread::JoinHandle<io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut captured = Vec::with_capacity(MAX_STREAM_BYTES);
        let mut buffer = [0u8; 8192];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            if captured.len() < MAX_STREAM_BYTES {
                let remaining = MAX_STREAM_BYTES - captured.len();
                captured.extend_from_slice(&buffer[..read.min(remaining)]);
            }
        }
        Ok(captured)
    })
}

fn join_capture(handle: thread::JoinHandle<io::Result<Vec<u8>>>) -> Result<Vec<u8>, RunnerError> {
    let captured = handle
        .join()
        .map_err(|_| RunnerError::Io(io::Error::other("verifier capture thread panicked")))??;
    Ok(captured)
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> io::Result<(ExitStatus, bool)> {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok((status, false)),
            Ok(None) if started.elapsed() >= timeout => {
                let terminate_result = terminate_child_tree(child);
                let wait_result = child.wait();
                terminate_result?;
                return wait_result.map(|status| (status, true));
            }
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(error) => {
                let _ = terminate_child_tree(child);
                let _ = child.wait();
                return Err(error);
            }
        }
    }
}

#[cfg(unix)]
fn terminate_child_tree(child: &mut Child) -> io::Result<()> {
    let pgid = i32::try_from(child.id())
        .map_err(|_| io::Error::other("verifier pid exceeds process-group range"))?;

    // SAFETY: the child was spawned as leader of a dedicated process group via
    // CommandExt::process_group(0). `kill` receives only the negated numeric PGID
    // and SIGKILL; it does not dereference pointers or transfer ownership.
    let result = unsafe { libc::kill(-pgid, libc::SIGKILL) };
    if result == 0 {
        return Ok(());
    }

    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        match child.try_wait()? {
            Some(_) => Ok(()),
            None => child.kill(),
        }
    } else {
        Err(error)
    }
}

#[cfg(not(unix))]
fn terminate_child_tree(child: &mut Child) -> io::Result<()> {
    child.kill()
}

fn confined_source(root: &Path, relative: &str) -> Result<PathBuf, RunnerError> {
    if !safe_relative(relative) {
        return Err(RunnerError::OverlayIntegrity(format!(
            "unsafe source path `{relative}`"
        )));
    }
    let source = root.join(relative);
    let canonical = fs::canonicalize(&source)
        .map_err(|error| RunnerError::OverlayIntegrity(format!("{}: {error}", source.display())))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        return Err(RunnerError::OverlayIntegrity(source.display().to_string()));
    }
    Ok(canonical)
}

fn confined_destination(root: &Path, relative: &str) -> Result<PathBuf, RunnerError> {
    if !safe_relative(relative) {
        return Err(RunnerError::UnsafeOverlayDestination(relative.into()));
    }
    let destination = root.join(relative);
    let mut candidate = Some(destination.as_path());
    while let Some(path) = candidate {
        match fs::symlink_metadata(path) {
            Ok(_) => {
                let canonical = fs::canonicalize(path)
                    .map_err(|_| RunnerError::UnsafeOverlayDestination(relative.into()))?;
                if !canonical.starts_with(root) {
                    return Err(RunnerError::UnsafeOverlayDestination(relative.into()));
                }
                return Ok(destination);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                candidate = path.parent();
            }
            Err(_) => return Err(RunnerError::UnsafeOverlayDestination(relative.into())),
        }
    }
    Err(RunnerError::UnsafeOverlayDestination(relative.into()))
}

fn confined_working_directory(root: &Path, relative: &str) -> Result<PathBuf, RunnerError> {
    if !safe_relative_allow_dot(relative) {
        return Err(RunnerError::UnsafeOverlayDestination(relative.into()));
    }
    let directory = root.join(relative);
    let canonical = fs::canonicalize(&directory)
        .map_err(|_| RunnerError::UnsafeOverlayDestination(relative.into()))?;
    if !canonical.starts_with(root) || !canonical.is_dir() {
        return Err(RunnerError::UnsafeOverlayDestination(relative.into()));
    }
    Ok(canonical)
}

fn safe_relative(value: &str) -> bool {
    value != "." && safe_relative_allow_dot(value)
}

fn safe_relative_allow_dot(value: &str) -> bool {
    if value == "." {
        return true;
    }
    let path = Path::new(value);
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
