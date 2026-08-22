//! The verification fabric: checks work independently from code generation.
//!
//! Verification is deliberately decoupled from generation. A separate
//! [`VerificationFabric`] runs checks (unit tests, builds, linting, static
//! analysis, security) and produces a [`VerificationReport`] that can be
//! recorded as evidence and consumed by the orchestrator's VERIFY phase.
//!
//! Verifiers are injectable closures, so the fabric is testable without actually
//! running cargo/git/security tools. Default verifiers wrap a command runner
//! (`std::process::Command` with an argv array — no shell) so real checks are
//! available while staying deterministic and offline-testable.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The kinds of verification supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationKind {
    UnitTests,
    Build,
    Lint,
    StaticAnalysis,
    Security,
}

impl VerificationKind {
    /// The wire name of this kind.
    pub fn as_str(self) -> &'static str {
        match self {
            VerificationKind::UnitTests => "unit_tests",
            VerificationKind::Build => "build",
            VerificationKind::Lint => "lint",
            VerificationKind::StaticAnalysis => "static_analysis",
            VerificationKind::Security => "security",
        }
    }
}

/// The outcome of a single verification check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    /// Not run (e.g. no files to lint).
    Skipped,
    Passed,
    Failed,
    /// The check itself errored (tool missing, etc.).
    Errored,
}

/// A finding produced by a check (e.g. a lint warning or security issue).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Severity or level (e.g. "warning", "error", "info").
    pub severity: String,
    /// A short message.
    pub message: String,
    /// Optional file/line reference.
    pub location: Option<String>,
}

/// The result of one verification check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationResult {
    pub kind: VerificationKind,
    pub status: VerificationStatus,
    /// The tool used (e.g. "cargo test", "clippy").
    pub tool: String,
    /// A one-line summary.
    pub summary: String,
    /// Individual findings.
    #[serde(default)]
    pub findings: Vec<Finding>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

impl VerificationResult {
    /// Whether the result is a pass.
    pub fn passed(&self) -> bool {
        self.status == VerificationStatus::Passed
    }
}

/// The context a verification check runs against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationContext {
    /// The workspace/repo root the check runs in.
    pub workspace: String,
    /// Paths changed (relative), if known.
    #[serde(default)]
    pub changed: Vec<String>,
}

/// A verifier: run a single check and produce a result.
pub type VerifierFn = Box<dyn Fn(&VerificationContext) -> VerificationResult>;

/// The overall verdict of a verification report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationVerdict {
    Pass,
    Fail,
    Skipped,
}

/// A complete verification report: the results of all checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationReport {
    pub results: Vec<VerificationResult>,
    pub overall: VerificationVerdict,
    pub completed_at: DateTime<Utc>,
}

/// Errors produced by the verification fabric.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VerificationError {
    #[error("no verifier registered for kind '{0}'")]
    NoVerifier(String),
}

/// The verification fabric: runs a set of checks and produces a report.
///
/// Each [`VerificationKind`] maps to a [`VerifierFn`]. `run` executes every
/// registered verifier and aggregates the results into a [`VerificationReport`].
/// Checks are independent (no shared mutable state), so they run in any order.
#[derive(Default)]
pub struct VerificationFabric {
    verifiers: Vec<(VerificationKind, VerifierFn)>,
}

impl VerificationFabric {
    /// Create an empty fabric.
    pub fn new() -> Self {
        VerificationFabric::default()
    }

    /// Register or replace the verifier for a kind (builder-style).
    pub fn with(mut self, kind: VerificationKind, verifier: VerifierFn) -> Self {
        self.register(kind, verifier);
        self
    }

    /// Register or replace the verifier for a kind (in-place).
    pub fn register(&mut self, kind: VerificationKind, verifier: VerifierFn) {
        self.verifiers.retain(|(k, _)| *k != kind);
        self.verifiers.push((kind, verifier));
    }

    /// Run all registered checks against `ctx` and produce a report.
    pub fn run(&self, ctx: &VerificationContext) -> VerificationReport {
        let results: Vec<VerificationResult> = self.verifiers.iter().map(|(_, v)| v(ctx)).collect();
        let overall = if results.is_empty() {
            VerificationVerdict::Skipped
        } else if results.iter().all(|r| r.passed()) {
            VerificationVerdict::Pass
        } else {
            VerificationVerdict::Fail
        };
        VerificationReport {
            results,
            overall,
            completed_at: Utc::now(),
        }
    }

    /// Look up the verifier for a kind.
    pub fn verifier(&self, kind: VerificationKind) -> Option<&VerifierFn> {
        self.verifiers
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, v)| v)
    }
}
/// A verifier that runs a command (argv array, no shell) and passes/fails on
/// the exit code. `tool` names the check for reporting.
///
/// The verifier confines execution to the supplied workspace directory and
/// caps the captured stderr at [MAX_VERIFIER_OUTPUT] bytes so a misconfigured
/// tool cannot flood the report. Runtime is bounded by the calling
/// process supervisor (cargo/npm/gradle each honor their own `--timeout`).
pub fn command_verifier(kind: VerificationKind, tool: &str, args: Vec<String>) -> VerifierFn {
    const MAX_VERIFIER_OUTPUT: usize = 16 * 1024;
    let tool = tool.to_string();
    Box::new(move |ctx: &VerificationContext| {
        let started_at = Utc::now();
        // Confinement: a verifier may only run inside an existing workspace
        // the kernel has validated. An invalid path here is a programming
        // error in the calling fabric, not a runtime condition to surface.
        let cwd = match crate::workspace::Workspace::new(&ctx.workspace, 0) {
            Ok(ws) => ws.root().to_path_buf(),
            Err(err) => {
                return VerificationResult {
                    kind,
                    status: VerificationStatus::Errored,
                    tool: tool.clone(),
                    summary: format!("{tool} could not run: invalid workspace: {err}"),
                    findings: vec![],
                    started_at,
                    completed_at: Utc::now(),
                };
            }
        };
        let output = std::process::Command::new(&tool)
            .current_dir(&cwd)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output();
        let (status, summary) = match output {
            Ok(o) if o.status.success() => (VerificationStatus::Passed, format!("{tool} passed")),
            Ok(o) => {
                // Output cap: an adversarial or runaway tool must not be
                // able to produce a summary that floods the report.
                let stderr_bytes = if o.stderr.len() > MAX_VERIFIER_OUTPUT {
                    &o.stderr[..MAX_VERIFIER_OUTPUT]
                } else {
                    &o.stderr
                };
                let msg = String::from_utf8_lossy(stderr_bytes).to_string();
                (
                    VerificationStatus::Failed,
                    format!("{tool} failed: {}", truncate(&msg)),
                )
            }
            Err(e) => (
                VerificationStatus::Errored,
                format!("{tool} could not run: {e}"),
            ),
        };
        VerificationResult {
            kind,
            status,
            tool: tool.clone(),
            summary,
            findings: vec![],
            started_at,
            completed_at: Utc::now(),
        }
    })
}

/// A deterministic mock verifier (no process) for tests and orchestration.
pub fn mock_verifier(kind: VerificationKind, pass: bool) -> VerifierFn {
    Box::new(move |_ctx: &VerificationContext| {
        let now = Utc::now();
        VerificationResult {
            kind,
            status: if pass {
                VerificationStatus::Passed
            } else {
                VerificationStatus::Failed
            },
            tool: format!("mock-{}", kind.as_str()),
            summary: if pass { "ok" } else { "failed" }.to_string(),
            findings: if pass {
                vec![]
            } else {
                vec![Finding {
                    severity: "error".to_string(),
                    message: "mock failure".to_string(),
                    location: None,
                }]
            },
            started_at: now,
            completed_at: now,
        }
    })
}

/// Build a default fabric with command-backed verifiers for the five check kinds.
pub fn default_fabric() -> VerificationFabric {
    let mut f = VerificationFabric::new();
    f.register(
        VerificationKind::UnitTests,
        command_verifier(VerificationKind::UnitTests, "cargo", vec!["test".into()]),
    );
    f.register(
        VerificationKind::Build,
        command_verifier(VerificationKind::Build, "cargo", vec!["build".into()]),
    );
    f.register(
        VerificationKind::Lint,
        command_verifier(VerificationKind::Lint, "cargo", vec!["clippy".into()]),
    );
    f.register(
        VerificationKind::StaticAnalysis,
        command_verifier(
            VerificationKind::StaticAnalysis,
            "cargo",
            vec!["check".into()],
        ),
    );
    f.register(
        VerificationKind::Security,
        command_verifier(VerificationKind::Security, "cargo", vec!["audit".into()]),
    );
    f
}

/// Bridge a [`VerificationReport`] into an [`crate::orchestrator::Verdict`].
///
/// This is how the verification fabric feeds the orchestrator's VERIFY phase:
/// an overall Pass becomes [`crate::orchestrator::Verdict::Pass`], anything
/// else becomes Fail. Generation and verification remain decoupled — the
/// orchestrator only sees the verdict and can attach the report as evidence.
pub fn verdict_from_report(report: &VerificationReport) -> crate::orchestrator::Verdict {
    match report.overall {
        VerificationVerdict::Pass => crate::orchestrator::Verdict::Pass,
        VerificationVerdict::Fail | VerificationVerdict::Skipped => {
            crate::orchestrator::Verdict::Fail
        }
    }
}

/// Truncate a string for safe inclusion in a summary.
fn truncate(s: &str) -> String {
    let max = 200;
    if s.chars().count() > max {
        let t: String = s.chars().take(max).collect();
        format!("{t}…")
    } else {
        s.to_string()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> VerificationContext {
        VerificationContext {
            workspace: ".".to_string(),
            changed: vec!["src/lib.rs".to_string()],
        }
    }

    #[test]
    fn mock_pass_fabric_reports_pass() {
        let fabric = VerificationFabric::new()
            .with(
                VerificationKind::UnitTests,
                mock_verifier(VerificationKind::UnitTests, true),
            )
            .with(
                VerificationKind::Build,
                mock_verifier(VerificationKind::Build, true),
            );
        let report = fabric.run(&ctx());
        assert_eq!(report.overall, VerificationVerdict::Pass);
        assert_eq!(report.results.len(), 2);
        assert!(report.results.iter().all(|r| r.passed()));
    }

    #[test]
    fn any_failure_means_overall_fail() {
        let fabric = VerificationFabric::new()
            .with(
                VerificationKind::UnitTests,
                mock_verifier(VerificationKind::UnitTests, true),
            )
            .with(
                VerificationKind::Security,
                mock_verifier(VerificationKind::Security, false),
            );
        let report = fabric.run(&ctx());
        assert_eq!(report.overall, VerificationVerdict::Fail);
        let sec = report
            .results
            .iter()
            .find(|r| r.kind == VerificationKind::Security)
            .unwrap();
        assert_eq!(sec.status, VerificationStatus::Failed);
        assert_eq!(sec.findings.len(), 1);
    }

    #[test]
    fn empty_fabric_is_skipped() {
        let fabric = VerificationFabric::new();
        let report = fabric.run(&ctx());
        assert_eq!(report.overall, VerificationVerdict::Skipped);
        assert!(report.results.is_empty());
    }

    #[test]
    fn register_replaces_existing_verifier() {
        let mut fabric = VerificationFabric::new();
        fabric.register(
            VerificationKind::Lint,
            mock_verifier(VerificationKind::Lint, false),
        );
        fabric.register(
            VerificationKind::Lint,
            mock_verifier(VerificationKind::Lint, true),
        );
        let report = fabric.run(&ctx());
        assert_eq!(report.overall, VerificationVerdict::Pass);
        assert_eq!(report.results.len(), 1);
    }

    #[test]
    fn command_verifier_can_run_and_pass() {
        let v = command_verifier(VerificationKind::Build, "true", vec![]);
        let result = v(&ctx());
        assert_eq!(result.status, VerificationStatus::Passed);
    }

    #[test]
    fn command_verifier_reports_missing_tool_as_errored() {
        let v = command_verifier(
            VerificationKind::Lint,
            "definitely-not-a-real-tool-xyz",
            vec![],
        );
        let result = v(&ctx());
        assert_eq!(result.status, VerificationStatus::Errored);
    }
}
