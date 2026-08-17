use std::path::{Path, PathBuf};

use forge_core::{derive_outcome, EvalAttempt, EvalOutcome, EvalTask, SafetyFinding};

use crate::{
    apply_verifier_overlays, changed_paths, materialize_checkout, run_verifier, EvalFixture,
    RunnerError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptMetadata {
    pub attempts: u32,
    pub elapsed_ms: u64,
    pub tool_calls: Option<u32>,
    pub intervention_count: Option<u32>,
}

pub trait AttemptDriver {
    fn run(
        &mut self,
        task: &EvalTask,
        workspace: &Path,
    ) -> Result<AttemptMetadata, RunnerError>;
}

pub struct EvaluationRunner<D> {
    driver: D,
    crate_root: PathBuf,
}

impl<D> EvaluationRunner<D> {
    pub fn new(driver: D, crate_root: impl AsRef<Path>) -> Self {
        Self {
            driver,
            crate_root: crate_root.as_ref().to_path_buf(),
        }
    }
}

impl<D: AttemptDriver> EvaluationRunner<D> {
    pub fn evaluate(
        &mut self,
        fixture: &EvalFixture,
        source_repo: &Path,
    ) -> Result<EvalOutcome, RunnerError> {
        fixture.task.validate()?;

        let checkout = match materialize_checkout(source_repo, &fixture.task.base_sha) {
            Ok(checkout) => checkout,
            Err(error) => {
                return infrastructure_outcome(
                    &fixture.task,
                    AttemptMetadata {
                        attempts: 0,
                        elapsed_ms: 0,
                        tool_calls: None,
                        intervention_count: None,
                    },
                    vec![],
                    vec![],
                    error,
                )
            }
        };

        let metadata = match self.driver.run(&fixture.task, checkout.path()) {
            Ok(metadata) => metadata,
            Err(error) => {
                return infrastructure_outcome(
                    &fixture.task,
                    AttemptMetadata {
                        attempts: 0,
                        elapsed_ms: 0,
                        tool_calls: None,
                        intervention_count: None,
                    },
                    vec![],
                    vec![],
                    error,
                )
            }
        };

        let changed = match changed_paths(checkout.path()) {
            Ok(changed) => changed,
            Err(error) => {
                return infrastructure_outcome(
                    &fixture.task,
                    metadata,
                    vec![],
                    vec![],
                    error,
                )
            }
        };
        let safety_findings = overlay_collisions(&changed, fixture);

        if let Err(error) = apply_verifier_overlays(
            &self.crate_root,
            checkout.path(),
            &fixture.verifier_overlay,
        ) {
            return infrastructure_outcome(
                &fixture.task,
                metadata,
                changed,
                safety_findings,
                error,
            );
        }

        let executions = match run_verifier(checkout.path(), &fixture.task.verifier) {
            Ok(executions) => executions,
            Err(error) => {
                return infrastructure_outcome(
                    &fixture.task,
                    metadata,
                    changed,
                    safety_findings,
                    error,
                )
            }
        };
        let verifier_elapsed = executions.iter().map(|execution| execution.elapsed_ms).sum();
        let evidence = executions
            .into_iter()
            .map(|execution| execution.evidence)
            .collect();

        derive_outcome(
            &fixture.task,
            EvalAttempt {
                task_key: fixture.task.key()?,
                attempts: metadata.attempts,
                verifier_evidence: evidence,
                changed_paths: changed,
                safety_findings,
                elapsed_ms: metadata.elapsed_ms.saturating_add(verifier_elapsed),
                tool_calls: metadata.tool_calls,
                intervention_count: metadata.intervention_count,
                infrastructure_error: None,
            },
        )
        .map_err(RunnerError::from)
    }
}

fn overlay_collisions(changed_paths: &[String], fixture: &EvalFixture) -> Vec<SafetyFinding> {
    let mut findings = Vec::new();
    for overlay in &fixture.verifier_overlay {
        if changed_paths
            .iter()
            .any(|path| path == &overlay.destination_path)
        {
            findings.push(SafetyFinding {
                kind: "verifier_overlay_collision".into(),
                path: Some(overlay.destination_path.clone()),
                detail: "evaluated attempt modified an independent verifier destination".into(),
            });
        }
    }
    findings.sort();
    findings.dedup();
    findings
}

fn infrastructure_outcome(
    task: &EvalTask,
    metadata: AttemptMetadata,
    changed_paths: Vec<String>,
    safety_findings: Vec<SafetyFinding>,
    error: RunnerError,
) -> Result<EvalOutcome, RunnerError> {
    derive_outcome(
        task,
        EvalAttempt {
            task_key: task.key()?,
            attempts: metadata.attempts,
            verifier_evidence: vec![],
            changed_paths,
            safety_findings,
            elapsed_ms: metadata.elapsed_ms,
            tool_calls: metadata.tool_calls,
            intervention_count: metadata.intervention_count,
            infrastructure_error: Some(error.to_string()),
        },
    )
    .map_err(RunnerError::from)
}
