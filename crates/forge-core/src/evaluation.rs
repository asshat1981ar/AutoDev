//! Pure evaluation-domain contracts for AutoDev's historical self-evaluation corpus.
//!
//! This module is deliberately side-effect free. It validates task definitions and
//! computes deterministic identities; it does not execute Git, processes, models,
//! policy, or agent actions.

use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::evidence::sha256_hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskSourceKind {
    Commit,
    MergedPullRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskSource {
    pub kind: TaskSourceKind,
    pub repository: String,
    pub source_ref: String,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierStep {
    pub id: String,
    pub program: String,
    pub args: Vec<String>,
    pub working_directory: String,
    pub timeout_seconds: u32,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationRecipe {
    pub steps: Vec<VerifierStep>,
    #[serde(default)]
    pub asset_fingerprints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectedSurface {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalTask {
    pub id: String,
    pub source: TaskSource,
    pub base_sha: String,
    pub specification: String,
    pub acceptance_criteria: Vec<String>,
    pub verifier: VerificationRecipe,
    pub protected: ProtectedSurface,
    pub expected_change_scope: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EvalTaskKey {
    pub task_id: String,
    pub task_fingerprint: String,
    pub verifier_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EvaluationError {
    #[error("field `{0}` must not be empty")]
    EmptyField(&'static str),
    #[error("task `{0}` must contain at least one verifier step")]
    EmptyVerifier(String),
    #[error("field `{field}` contains invalid full git SHA `{value}`")]
    InvalidGitSha { field: &'static str, value: String },
    #[error("verifier asset fingerprint `{0}` is not a full SHA-256 digest")]
    InvalidVerifierAssetFingerprint(String),
    #[error("verifier step `{0}` must have a positive timeout")]
    InvalidTimeout(String),
    #[error("unsafe relative path `{path}` in {field}")]
    UnsafePath { field: &'static str, path: String },
    #[error("verifier step `{step_id}` uses an opaque shell wrapper")]
    OpaqueShell { step_id: String },
    #[error("protected path `{protected}` overlaps expected change scope `{expected}`")]
    ProtectedScopeOverlap { protected: String, expected: String },
    #[error("duplicate task id `{0}`")]
    DuplicateTaskId(String),
    #[error("attempt task key does not match task `{0}`")]
    TaskKeyMismatch(String),
    #[error("required verifier evidence is incomplete for task `{0}`")]
    IncompleteVerifierEvidence(String),
    #[error("report revision must not be empty")]
    EmptyRevision,
}

impl EvalTask {
    pub fn validate(&self) -> Result<(), EvaluationError> {
        required(&self.id, "id")?;
        required(&self.source.repository, "source.repository")?;
        required(&self.specification, "specification")?;

        if !full_git_sha(&self.base_sha) {
            return Err(EvaluationError::InvalidGitSha {
                field: "base_sha",
                value: self.base_sha.clone(),
            });
        }
        if !full_git_sha(&self.source.source_ref) {
            return Err(EvaluationError::InvalidGitSha {
                field: "source_ref",
                value: self.source.source_ref.clone(),
            });
        }
        if self.verifier.steps.is_empty() {
            return Err(EvaluationError::EmptyVerifier(self.id.clone()));
        }

        for fingerprint in &self.verifier.asset_fingerprints {
            if !full_sha256(fingerprint) {
                return Err(EvaluationError::InvalidVerifierAssetFingerprint(
                    fingerprint.clone(),
                ));
            }
        }

        for step in &self.verifier.steps {
            required(&step.id, "verifier.step.id")?;
            required(&step.program, "verifier.step.program")?;
            if step.timeout_seconds == 0 {
                return Err(EvaluationError::InvalidTimeout(step.id.clone()));
            }
            if !safe_relative(&step.working_directory) {
                return Err(EvaluationError::UnsafePath {
                    field: "working_directory",
                    path: step.working_directory.clone(),
                });
            }
            if opaque_shell(step) {
                return Err(EvaluationError::OpaqueShell {
                    step_id: step.id.clone(),
                });
            }
        }

        for path in &self.protected.paths {
            if !safe_relative(path) {
                return Err(EvaluationError::UnsafePath {
                    field: "protected.paths",
                    path: path.clone(),
                });
            }
        }
        for path in &self.expected_change_scope {
            if !safe_relative(path) {
                return Err(EvaluationError::UnsafePath {
                    field: "expected_change_scope",
                    path: path.clone(),
                });
            }
        }
        for protected in &self.protected.paths {
            for expected in &self.expected_change_scope {
                if rules_overlap(protected, expected) {
                    return Err(EvaluationError::ProtectedScopeOverlap {
                        protected: protected.clone(),
                        expected: expected.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn task_fingerprint(&self) -> Result<String, EvaluationError> {
        self.validate()?;
        let mut normalized = self.clone();
        sort_dedup(&mut normalized.acceptance_criteria);
        sort_dedup(&mut normalized.protected.paths);
        sort_dedup(&mut normalized.expected_change_scope);
        sort_dedup(&mut normalized.verifier.asset_fingerprints);
        let bytes = serde_json::to_vec(&normalized).expect("validated EvalTask serializes");
        Ok(sha256_hex(&bytes))
    }

    pub fn verifier_fingerprint(&self) -> Result<String, EvaluationError> {
        self.validate()?;
        let mut normalized = self.verifier.clone();
        sort_dedup(&mut normalized.asset_fingerprints);
        let bytes =
            serde_json::to_vec(&normalized).expect("validated VerificationRecipe serializes");
        Ok(sha256_hex(&bytes))
    }

    pub fn key(&self) -> Result<EvalTaskKey, EvaluationError> {
        Ok(EvalTaskKey {
            task_id: self.id.clone(),
            task_fingerprint: self.task_fingerprint()?,
            verifier_fingerprint: self.verifier_fingerprint()?,
        })
    }
}

fn required(value: &str, field: &'static str) -> Result<(), EvaluationError> {
    if value.trim().is_empty() {
        Err(EvaluationError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn full_git_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn full_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn safe_relative(value: &str) -> bool {
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

fn opaque_shell(step: &VerifierStep) -> bool {
    let program = Path::new(&step.program)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(&step.program)
        .to_ascii_lowercase();
    let is_shell = matches!(
        program.as_str(),
        "sh" | "bash" | "zsh" | "pwsh" | "powershell" | "cmd" | "cmd.exe"
    );
    is_shell
        && step
            .args
            .iter()
            .any(|arg| matches!(arg.to_ascii_lowercase().as_str(), "-c" | "/c" | "-command"))
}

fn rules_overlap(left: &str, right: &str) -> bool {
    let left_prefix = left.ends_with('/');
    let right_prefix = right.ends_with('/');
    match (left_prefix, right_prefix) {
        (false, false) => left == right,
        (true, false) => right.starts_with(left),
        (false, true) => left.starts_with(right),
        (true, true) => left.starts_with(right) || right.starts_with(left),
    }
}

fn sort_dedup(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
}
