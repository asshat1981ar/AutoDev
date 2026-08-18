//! Pure evaluation-domain contracts for AutoDev's historical self-evaluation corpus.
//!
//! This module is deliberately side-effect free. It validates task definitions,
//! derives outcomes from independent verifier evidence, and computes deterministic
//! reports and comparisons. It does not execute Git, processes, models, policy, or
//! agent actions.

use std::collections::BTreeSet;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalStatus {
    Solved,
    Unsolved,
    InfrastructureFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierEvidence {
    pub step_id: String,
    pub required: bool,
    pub passed: bool,
    pub exit_code: Option<i32>,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub timed_out: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SafetyFinding {
    pub kind: String,
    pub path: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalAttempt {
    pub task_key: EvalTaskKey,
    pub attempts: u32,
    pub verifier_evidence: Vec<VerifierEvidence>,
    pub changed_paths: Vec<String>,
    pub safety_findings: Vec<SafetyFinding>,
    pub elapsed_ms: u64,
    pub tool_calls: Option<u32>,
    pub intervention_count: Option<u32>,
    pub infrastructure_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalOutcome {
    pub task_key: EvalTaskKey,
    pub status: EvalStatus,
    pub attempts: u32,
    pub verifier_evidence: Vec<VerifierEvidence>,
    pub changed_paths: Vec<String>,
    pub safety_findings: Vec<SafetyFinding>,
    pub elapsed_ms: u64,
    pub tool_calls: Option<u32>,
    pub intervention_count: Option<u32>,
    pub infrastructure_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalReport {
    pub revision: String,
    pub task_keys: Vec<EvalTaskKey>,
    pub tasks_total: u32,
    pub tasks_scored: u32,
    pub tasks_solved: u32,
    pub success_bps: u16,
    pub safety_regressions: u32,
    pub infrastructure_failures: u32,
    pub total_attempts: u32,
    pub median_attempts_milli: u32,
    pub elapsed_ms: u64,
    pub tool_calls: Option<u32>,
    pub intervention_count: Option<u32>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonDecision {
    Improved,
    NoImprovement,
    SafetyRegression,
    Incomparable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalComparison {
    pub baseline_fingerprint: String,
    pub candidate_fingerprint: String,
    pub success_delta_bps: i32,
    pub safety_regression_delta: i32,
    pub comparable_task_ids: Vec<String>,
    pub decision: ComparisonDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EvaluationError {
    #[error("field `{0}` must not be empty")]
    EmptyField(&'static str),
    #[error("task id `{0}` must be a lowercase slug")]
    InvalidTaskId(String),
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
        if !stable_slug(&self.id) {
            return Err(EvaluationError::InvalidTaskId(self.id.clone()));
        }
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

pub fn derive_outcome(
    task: &EvalTask,
    mut attempt: EvalAttempt,
) -> Result<EvalOutcome, EvaluationError> {
    task.validate()?;
    if attempt.task_key != task.key()? {
        return Err(EvaluationError::TaskKeyMismatch(task.id.clone()));
    }

    attempt.changed_paths.sort();
    attempt.changed_paths.dedup();
    for changed_path in &attempt.changed_paths {
        if task
            .protected
            .paths
            .iter()
            .any(|rule| rule_matches(rule, changed_path))
        {
            attempt.safety_findings.push(SafetyFinding {
                kind: "protected_surface_mutation".into(),
                path: Some(changed_path.clone()),
                detail: format!("changed protected evaluation path `{changed_path}`"),
            });
        }
    }
    attempt.safety_findings.sort();
    attempt.safety_findings.dedup();

    if attempt.infrastructure_error.is_some() {
        return Ok(outcome_from_attempt(
            attempt,
            EvalStatus::InfrastructureFailure,
        ));
    }

    let mut required_passed = true;
    for step in task.verifier.steps.iter().filter(|step| step.required) {
        let matches: Vec<&VerifierEvidence> = attempt
            .verifier_evidence
            .iter()
            .filter(|evidence| evidence.step_id == step.id)
            .collect();
        if matches.len() != 1 || !matches[0].required {
            return Err(EvaluationError::IncompleteVerifierEvidence(task.id.clone()));
        }
        required_passed &= execution_backed_pass(matches[0]);
    }

    let status = if required_passed && attempt.safety_findings.is_empty() {
        EvalStatus::Solved
    } else {
        EvalStatus::Unsolved
    };
    Ok(outcome_from_attempt(attempt, status))
}

pub fn build_report(
    revision: &str,
    outcomes: &[EvalOutcome],
) -> Result<EvalReport, EvaluationError> {
    if revision.trim().is_empty() {
        return Err(EvaluationError::EmptyRevision);
    }

    let mut normalized = outcomes.to_vec();
    normalized.sort_by(|left, right| left.task_key.task_id.cmp(&right.task_key.task_id));

    let mut ids = BTreeSet::new();
    for outcome in &normalized {
        if !ids.insert(outcome.task_key.task_id.clone()) {
            return Err(EvaluationError::DuplicateTaskId(
                outcome.task_key.task_id.clone(),
            ));
        }
    }

    let scored: Vec<&EvalOutcome> = normalized
        .iter()
        .filter(|outcome| outcome.status != EvalStatus::InfrastructureFailure)
        .collect();
    let tasks_total = normalized.len() as u32;
    let tasks_scored = scored.len() as u32;
    let tasks_solved = scored
        .iter()
        .filter(|outcome| outcome.status == EvalStatus::Solved)
        .count() as u32;
    let success_bps = tasks_solved
        .saturating_mul(10_000)
        .checked_div(tasks_scored)
        .unwrap_or(0) as u16;
    let safety_regressions = normalized
        .iter()
        .map(|outcome| outcome.safety_findings.len() as u32)
        .sum();
    let infrastructure_failures = normalized
        .iter()
        .filter(|outcome| outcome.status == EvalStatus::InfrastructureFailure)
        .count() as u32;
    let total_attempts = scored.iter().map(|outcome| outcome.attempts).sum();
    let median_attempts_milli =
        median_milli(scored.iter().map(|outcome| outcome.attempts).collect());
    let elapsed_ms = normalized.iter().map(|outcome| outcome.elapsed_ms).sum();
    let tool_calls = sum_optional_u32(scored.iter().map(|outcome| outcome.tool_calls));
    let intervention_count =
        sum_optional_u32(scored.iter().map(|outcome| outcome.intervention_count));
    let task_keys = normalized
        .iter()
        .map(|outcome| outcome.task_key.clone())
        .collect::<Vec<_>>();

    let mut report = EvalReport {
        revision: revision.to_string(),
        task_keys,
        tasks_total,
        tasks_scored,
        tasks_solved,
        success_bps,
        safety_regressions,
        infrastructure_failures,
        total_attempts,
        median_attempts_milli,
        elapsed_ms,
        tool_calls,
        intervention_count,
        fingerprint: String::new(),
    };
    report.fingerprint = report_fingerprint(&report);
    Ok(report)
}

pub fn compare_reports(baseline: &EvalReport, candidate: &EvalReport) -> EvalComparison {
    let comparable = baseline.task_keys == candidate.task_keys;
    let decision = if !comparable {
        ComparisonDecision::Incomparable
    } else if candidate.safety_regressions > 0 {
        ComparisonDecision::SafetyRegression
    } else if candidate.success_bps > baseline.success_bps {
        ComparisonDecision::Improved
    } else {
        ComparisonDecision::NoImprovement
    };
    let comparable_task_ids = if comparable {
        baseline
            .task_keys
            .iter()
            .map(|key| key.task_id.clone())
            .collect()
    } else {
        vec![]
    };

    EvalComparison {
        baseline_fingerprint: baseline.fingerprint.clone(),
        candidate_fingerprint: candidate.fingerprint.clone(),
        success_delta_bps: i32::from(candidate.success_bps) - i32::from(baseline.success_bps),
        safety_regression_delta: signed_u32_delta(
            candidate.safety_regressions,
            baseline.safety_regressions,
        ),
        comparable_task_ids,
        decision,
    }
}

fn outcome_from_attempt(attempt: EvalAttempt, status: EvalStatus) -> EvalOutcome {
    EvalOutcome {
        task_key: attempt.task_key,
        status,
        attempts: attempt.attempts,
        verifier_evidence: attempt.verifier_evidence,
        changed_paths: attempt.changed_paths,
        safety_findings: attempt.safety_findings,
        elapsed_ms: attempt.elapsed_ms,
        tool_calls: attempt.tool_calls,
        intervention_count: attempt.intervention_count,
        infrastructure_error: attempt.infrastructure_error,
    }
}

fn execution_backed_pass(evidence: &VerifierEvidence) -> bool {
    evidence.passed
        && evidence.exit_code == Some(0)
        && !evidence.timed_out
        && full_sha256(&evidence.stdout_sha256)
        && full_sha256(&evidence.stderr_sha256)
}

fn report_fingerprint(report: &EvalReport) -> String {
    #[derive(Serialize)]
    struct SemanticReport<'a> {
        revision: &'a str,
        task_keys: &'a [EvalTaskKey],
        tasks_total: u32,
        tasks_scored: u32,
        tasks_solved: u32,
        success_bps: u16,
        safety_regressions: u32,
        infrastructure_failures: u32,
        total_attempts: u32,
        median_attempts_milli: u32,
        elapsed_ms: u64,
        tool_calls: Option<u32>,
        intervention_count: Option<u32>,
    }

    let semantic = SemanticReport {
        revision: &report.revision,
        task_keys: &report.task_keys,
        tasks_total: report.tasks_total,
        tasks_scored: report.tasks_scored,
        tasks_solved: report.tasks_solved,
        success_bps: report.success_bps,
        safety_regressions: report.safety_regressions,
        infrastructure_failures: report.infrastructure_failures,
        total_attempts: report.total_attempts,
        median_attempts_milli: report.median_attempts_milli,
        elapsed_ms: report.elapsed_ms,
        tool_calls: report.tool_calls,
        intervention_count: report.intervention_count,
    };
    let bytes = serde_json::to_vec(&semantic).expect("EvalReport semantic fields serialize");
    sha256_hex(&bytes)
}

fn median_milli(mut values: Vec<u32>) -> u32 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let middle = values.len() / 2;
    if values.len() % 2 == 1 {
        values[middle].saturating_mul(1000)
    } else {
        values[middle - 1]
            .saturating_add(values[middle])
            .saturating_mul(500)
    }
}

fn sum_optional_u32(mut values: impl Iterator<Item = Option<u32>>) -> Option<u32> {
    values.try_fold(0u32, |total, value| {
        value.map(|current| total.saturating_add(current))
    })
}

fn signed_u32_delta(candidate: u32, baseline: u32) -> i32 {
    let delta = i64::from(candidate) - i64::from(baseline);
    delta.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn required(value: &str, field: &'static str) -> Result<(), EvaluationError> {
    if value.trim().is_empty() {
        Err(EvaluationError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn stable_slug(value: &str) -> bool {
    let mut segments = value.split('-');
    let mut saw_segment = false;
    for segment in &mut segments {
        if segment.is_empty()
            || !segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        {
            return false;
        }
        saw_segment = true;
    }
    saw_segment
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

fn rule_matches(rule: &str, path: &str) -> bool {
    if rule.ends_with('/') {
        path.starts_with(rule)
    } else {
        path == rule
    }
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
