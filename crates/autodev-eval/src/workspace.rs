use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Output};

use crate::RunnerError;

#[derive(Debug)]
pub struct IsolatedCheckout {
    root: tempfile::TempDir,
}

impl IsolatedCheckout {
    pub fn path(&self) -> &Path {
        self.root.path()
    }
}

pub fn materialize_checkout(
    source_repo: impl AsRef<Path>,
    sha: &str,
) -> Result<IsolatedCheckout, RunnerError> {
    if !full_git_sha(sha) {
        return Err(RunnerError::Git(format!("invalid full git SHA `{sha}`")));
    }

    let root = tempfile::tempdir()?;
    run_git(
        [
            "clone",
            "--no-hardlinks",
            "--no-checkout",
            path_str(source_repo.as_ref())?,
            path_str(root.path())?,
        ],
        "clone isolated evaluation checkout",
    )?;
    run_git_in(
        root.path(),
        ["checkout", "--detach", sha],
        "checkout pinned evaluation revision",
    )?;

    let head = output_utf8(run_git_in(
        root.path(),
        ["rev-parse", "HEAD"],
        "resolve evaluation checkout HEAD",
    )?)?;
    if head.trim() != sha {
        return Err(RunnerError::Git(format!(
            "materialized HEAD `{}` does not match requested SHA `{sha}`",
            head.trim()
        )));
    }

    let checkout = IsolatedCheckout { root };
    if !changed_paths(checkout.path())?.is_empty() {
        return Err(RunnerError::Git(
            "new evaluation checkout is unexpectedly dirty".into(),
        ));
    }
    Ok(checkout)
}

pub fn changed_paths(workspace: impl AsRef<Path>) -> Result<Vec<String>, RunnerError> {
    let output = run_git_in(
        workspace.as_ref(),
        ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        "inspect evaluation workspace status",
    )?;
    parse_porcelain_z(&output.stdout)
}

fn run_git<const N: usize>(args: [&str; N], label: &str) -> Result<Output, RunnerError> {
    let mut command = Command::new("git");
    command.args(args);
    run_command(command, label)
}

fn run_git_in<const N: usize>(
    repo: &Path,
    args: [&str; N],
    label: &str,
) -> Result<Output, RunnerError> {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo).args(args);
    run_command(command, label)
}

fn run_command(mut command: Command, label: &str) -> Result<Output, RunnerError> {
    let output = command.output().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            RunnerError::MissingExecutable("git".into())
        } else {
            RunnerError::Io(error)
        }
    })?;
    if output.status.success() {
        return Ok(output);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(RunnerError::Git(format!(
        "{label} exited with {}{}",
        output.status,
        if stderr.is_empty() {
            String::new()
        } else {
            format!(": {stderr}")
        }
    )))
}

fn parse_porcelain_z(bytes: &[u8]) -> Result<Vec<String>, RunnerError> {
    let records: Vec<&[u8]> = bytes
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
        .collect();
    let mut paths = BTreeSet::new();
    let mut index = 0;

    while index < records.len() {
        let record = records[index];
        if record.len() < 4 || record[2] != b' ' {
            return Err(RunnerError::Git(
                "malformed git status porcelain record".into(),
            ));
        }
        let status_x = record[0];
        let status_y = record[1];
        let path = std::str::from_utf8(&record[3..])
            .map_err(|_| RunnerError::Git("git status returned a non-UTF-8 path".into()))?;
        paths.insert(path.to_string());

        if matches!(status_x, b'R' | b'C') || matches!(status_y, b'R' | b'C') {
            index += 1;
            if index >= records.len() {
                return Err(RunnerError::Git(
                    "git status rename/copy record is missing its source path".into(),
                ));
            }
        }
        index += 1;
    }

    Ok(paths.into_iter().collect())
}

fn output_utf8(output: Output) -> Result<String, RunnerError> {
    String::from_utf8(output.stdout)
        .map_err(|_| RunnerError::Git("git returned non-UTF-8 output".into()))
}

fn path_str(path: &Path) -> Result<&str, RunnerError> {
    path.to_str()
        .ok_or_else(|| RunnerError::Git("evaluation path is not valid UTF-8".into()))
}

fn full_git_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
