//! Capability-gated trusted native plugin execution.
//!
//! Native plugins run in-process and therefore retain the ambient authority of
//! ForgeCore. This module validates host policy and bounds returned data, but it
//! is not a sandbox for untrusted code. Untrusted plugins require a process or
//! WASM boundary with independently enforced memory, time, filesystem, process,
//! and network limits.

use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::evidence::{ExecutionResult, ExecutionStatus};

/// Limits enforced around a plugin invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginLimits {
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    /// Maximum serialized bytes for findings, diagnostics, and artifact names.
    pub max_metadata_bytes: u64,
    pub max_findings: u32,
    pub max_artifacts: u32,
    pub max_artifact_bytes: u64,
}

impl Default for PluginLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 1_048_576,
            max_output_bytes: 1_048_576,
            max_metadata_bytes: 1_048_576,
            max_findings: 1_000,
            max_artifacts: 100,
            max_artifact_bytes: 10_485_760,
        }
    }
}

/// Input provided to a plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRequest {
    pub plugin_id: String,
    pub task_id: String,
    pub action_id: String,
    pub input: serde_json::Value,
}

/// Policy supplied by the trusted host for one plugin invocation.
///
/// Keeping this separate from [`PluginRequest`] prevents deserialized plugin
/// input from granting itself capabilities or relaxing resource limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginPolicy {
    pub granted_capabilities: Vec<String>,
    pub limits: PluginLimits,
}

/// A source location attached to a plugin finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginLocation {
    pub path: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

/// A diagnostic emitted by a plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginFinding {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub location: Option<PluginLocation>,
    pub remediation: Option<String>,
}

/// An artifact emitted by a plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginArtifact {
    pub name: String,
    pub content: Vec<u8>,
}

/// Measured usage from a successful invocation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginUsage {
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub findings: u32,
    pub artifacts: u32,
    pub artifact_bytes: u64,
    pub elapsed_ms: u64,
}

/// Output returned by a plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginResponse {
    pub output: serde_json::Value,
    pub findings: Vec<PluginFinding>,
    pub artifacts: Vec<PluginArtifact>,
    pub diagnostics: Vec<String>,
    #[serde(default)]
    pub usage: PluginUsage,
}

/// Errors raised by plugin validation or execution.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PluginError {
    #[error("plugin id must not be empty")]
    MissingPluginId,
    #[error("plugin task id must not be empty")]
    MissingTaskId,
    #[error("plugin action id must not be empty")]
    MissingActionId,
    #[error("plugin capability '{0}' was not granted")]
    CapabilityDenied(String),
    #[error("requested plugin '{requested}' does not match loaded plugin '{actual}'")]
    PluginIdMismatch { requested: String, actual: String },
    #[error("plugin input exceeds {actual} bytes; maximum is {maximum}")]
    InputTooLarge { actual: u64, maximum: u64 },
    #[error("plugin output exceeds {actual} bytes; maximum is {maximum}")]
    OutputTooLarge { actual: u64, maximum: u64 },
    #[error("plugin metadata exceeds {actual} bytes; maximum is {maximum}")]
    MetadataTooLarge { actual: u64, maximum: u64 },
    #[error("plugin emitted {actual} findings; maximum is {maximum}")]
    TooManyFindings { actual: u32, maximum: u32 },
    #[error("plugin emitted {actual} artifacts; maximum is {maximum}")]
    TooManyArtifacts { actual: u32, maximum: u32 },
    #[error("plugin artifacts contain {actual} bytes; maximum is {maximum}")]
    ArtifactBytesTooLarge { actual: u64, maximum: u64 },
    #[error("plugin panicked: {0}")]
    Panic(String),
    #[error("plugin output could not be serialized: {0}")]
    InvalidOutput(String),
}

/// A synchronous native plugin.
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn required_capabilities(&self) -> &[&str] {
        &[]
    }
    fn execute(&self, request: &PluginRequest) -> Result<PluginResponse, PluginError>;
}

/// Invoke a plugin after validating identity, capabilities, and resource bounds.
/// Panics are converted into structured errors and never escape this boundary.
pub fn execute_plugin(
    plugin: &dyn Plugin,
    request: PluginRequest,
    policy: &PluginPolicy,
) -> Result<PluginResponse, PluginError> {
    validate_request(plugin, &request, policy)?;
    let input_bytes = serialized_size(&request.input)?;
    if input_bytes > policy.limits.max_input_bytes {
        return Err(PluginError::InputTooLarge {
            actual: input_bytes,
            maximum: policy.limits.max_input_bytes,
        });
    }

    let started = Instant::now();
    let response = panic::catch_unwind(AssertUnwindSafe(|| plugin.execute(&request)))
        .map_err(|payload| PluginError::Panic(panic_message(payload)))??;
    validate_response(&response, &policy.limits)?;

    let output_bytes = serialized_size(&response.output)?;
    let artifact_bytes = artifact_bytes(&response.artifacts, policy.limits.max_artifact_bytes)?;
    let findings =
        u32::try_from(response.findings.len()).map_err(|_| PluginError::TooManyFindings {
            actual: u32::MAX,
            maximum: policy.limits.max_findings,
        })?;
    let artifacts =
        u32::try_from(response.artifacts.len()).map_err(|_| PluginError::TooManyArtifacts {
            actual: u32::MAX,
            maximum: policy.limits.max_artifacts,
        })?;
    let mut response = response;
    response.usage = PluginUsage {
        input_bytes,
        output_bytes,
        findings,
        artifacts,
        artifact_bytes,
        elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    };
    Ok(response)
}

/// Convert a plugin outcome into the common execution evidence shape.
///
/// This adapter deliberately does not persist evidence itself. Callers can pass
/// the returned result to [`crate::evidence::record_from`] and
/// [`crate::evidence::EvidenceStore`] just like any other execution adapter.
pub fn plugin_result_to_execution_result(
    action_id: &str,
    plugin_id: &str,
    started_at: DateTime<Utc>,
    outcome: Result<PluginResponse, PluginError>,
) -> ExecutionResult {
    let completed_at = Utc::now();
    match outcome {
        Ok(response) => ExecutionResult {
            action_id: action_id.to_string(),
            status: ExecutionStatus::Succeeded,
            started_at,
            completed_at,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            artifacts: response
                .artifacts
                .iter()
                .map(|artifact| artifact.name.clone())
                .collect(),
            verification: Some(serde_json::json!({
                "plugin_id": plugin_id,
                "output": response.output,
                "findings": response.findings,
                "diagnostics": response.diagnostics,
                "usage": response.usage,
            })),
            error: None,
        },
        Err(error) => ExecutionResult {
            action_id: action_id.to_string(),
            status: ExecutionStatus::Failed,
            started_at,
            completed_at,
            exit_code: None,
            stdout: String::new(),
            stderr: error.to_string(),
            artifacts: vec![],
            verification: Some(serde_json::json!({ "plugin_id": plugin_id })),
            error: Some(error.to_string()),
        },
    }
}

fn validate_request(
    plugin: &dyn Plugin,
    request: &PluginRequest,
    policy: &PluginPolicy,
) -> Result<(), PluginError> {
    if request.plugin_id.trim().is_empty() {
        return Err(PluginError::MissingPluginId);
    }
    if request.task_id.trim().is_empty() {
        return Err(PluginError::MissingTaskId);
    }
    if request.action_id.trim().is_empty() {
        return Err(PluginError::MissingActionId);
    }
    if request.plugin_id != plugin.id() {
        return Err(PluginError::PluginIdMismatch {
            requested: request.plugin_id.clone(),
            actual: plugin.id().to_string(),
        });
    }
    for required in plugin.required_capabilities() {
        if !policy
            .granted_capabilities
            .iter()
            .any(|granted| granted == required)
        {
            return Err(PluginError::CapabilityDenied((*required).to_string()));
        }
    }
    Ok(())
}

fn validate_response(response: &PluginResponse, limits: &PluginLimits) -> Result<(), PluginError> {
    let output_bytes = serialized_size(&response.output)?;
    if output_bytes > limits.max_output_bytes {
        return Err(PluginError::OutputTooLarge {
            actual: output_bytes,
            maximum: limits.max_output_bytes,
        });
    }
    let metadata_bytes = serialized_size(&serde_json::json!({
        "findings": &response.findings,
        "diagnostics": &response.diagnostics,
        "artifact_names": response
            .artifacts
            .iter()
            .map(|artifact| &artifact.name)
            .collect::<Vec<_>>(),
    }))?;
    if metadata_bytes > limits.max_metadata_bytes {
        return Err(PluginError::MetadataTooLarge {
            actual: metadata_bytes,
            maximum: limits.max_metadata_bytes,
        });
    }
    let findings =
        u32::try_from(response.findings.len()).map_err(|_| PluginError::TooManyFindings {
            actual: u32::MAX,
            maximum: limits.max_findings,
        })?;
    if findings > limits.max_findings {
        return Err(PluginError::TooManyFindings {
            actual: findings,
            maximum: limits.max_findings,
        });
    }
    let artifacts =
        u32::try_from(response.artifacts.len()).map_err(|_| PluginError::TooManyArtifacts {
            actual: u32::MAX,
            maximum: limits.max_artifacts,
        })?;
    if artifacts > limits.max_artifacts {
        return Err(PluginError::TooManyArtifacts {
            actual: artifacts,
            maximum: limits.max_artifacts,
        });
    }
    let artifact_bytes = artifact_bytes(&response.artifacts, limits.max_artifact_bytes)?;
    if artifact_bytes > limits.max_artifact_bytes {
        return Err(PluginError::ArtifactBytesTooLarge {
            actual: artifact_bytes,
            maximum: limits.max_artifact_bytes,
        });
    }
    Ok(())
}

fn artifact_bytes(artifacts: &[PluginArtifact], maximum: u64) -> Result<u64, PluginError> {
    artifacts.iter().try_fold(0_u64, |total, artifact| {
        let length = u64::try_from(artifact.content.len()).unwrap_or(u64::MAX);
        total
            .checked_add(length)
            .ok_or(PluginError::ArtifactBytesTooLarge {
                actual: u64::MAX,
                maximum,
            })
    })
}

fn serialized_size(value: &serde_json::Value) -> Result<u64, PluginError> {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len() as u64)
        .map_err(|error| PluginError::InvalidOutput(error.to_string()))
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    let message = payload
        .downcast_ref::<&str>()
        .map(|value| (*value).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic payload".to_string());
    message
        .replace(['\n', '\r'], " ")
        .chars()
        .take(512)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct EchoPlugin;
    impl Plugin for EchoPlugin {
        fn id(&self) -> &str {
            "echo"
        }
        fn required_capabilities(&self) -> &[&str] {
            &["source.read"]
        }
        fn execute(&self, request: &PluginRequest) -> Result<PluginResponse, PluginError> {
            Ok(PluginResponse {
                output: request.input.clone(),
                findings: vec![],
                artifacts: vec![],
                diagnostics: vec![],
                usage: PluginUsage::default(),
            })
        }
    }

    fn request() -> PluginRequest {
        PluginRequest {
            plugin_id: "echo".into(),
            task_id: "task".into(),
            action_id: "action".into(),
            input: json!({"ok": true}),
        }
    }

    fn policy() -> PluginPolicy {
        PluginPolicy {
            granted_capabilities: vec!["source.read".into()],
            limits: PluginLimits::default(),
        }
    }

    #[test]
    fn executes_and_records_usage() {
        let response = execute_plugin(&EchoPlugin, request(), &policy()).unwrap();
        assert_eq!(response.output, json!({"ok": true}));
        assert_eq!(response.usage.findings, 0);
        assert!(response.usage.input_bytes > 0);
    }

    #[test]
    fn denies_missing_capability() {
        let mut policy = policy();
        policy.granted_capabilities.clear();
        assert_eq!(
            execute_plugin(&EchoPlugin, request(), &policy),
            Err(PluginError::CapabilityDenied("source.read".into()))
        );
    }

    #[test]
    fn enforces_input_limit() {
        let mut request = request();
        request.input = json!("long input");
        let mut policy = policy();
        policy.limits.max_input_bytes = 1;
        assert!(matches!(
            execute_plugin(&EchoPlugin, request, &policy),
            Err(PluginError::InputTooLarge { .. })
        ));
    }

    struct PanicPlugin;
    impl Plugin for PanicPlugin {
        fn id(&self) -> &str {
            "panic"
        }
        fn execute(&self, _: &PluginRequest) -> Result<PluginResponse, PluginError> {
            panic!("boom\nsecret")
        }
    }

    #[test]
    fn converts_panics_to_bounded_errors() {
        let mut request = request();
        request.plugin_id = "panic".into();
        let error = execute_plugin(&PanicPlugin, request, &policy()).unwrap_err();
        assert_eq!(error, PluginError::Panic("boom secret".into()));
    }

    struct MetadataPlugin;
    impl Plugin for MetadataPlugin {
        fn id(&self) -> &str {
            "metadata"
        }

        fn execute(&self, _: &PluginRequest) -> Result<PluginResponse, PluginError> {
            Ok(PluginResponse {
                output: json!(null),
                findings: vec![],
                artifacts: vec![],
                diagnostics: vec!["x".repeat(128)],
                usage: PluginUsage::default(),
            })
        }
    }

    #[test]
    fn enforces_metadata_limit() {
        let mut request = request();
        request.plugin_id = "metadata".into();
        let mut policy = policy();
        policy.limits.max_metadata_bytes = 32;
        assert!(matches!(
            execute_plugin(&MetadataPlugin, request, &policy),
            Err(PluginError::MetadataTooLarge { .. })
        ));
    }

    #[test]
    fn reports_plugin_identity_mismatch_separately_from_capabilities() {
        let mut request = request();
        request.plugin_id = "other".into();
        assert_eq!(
            execute_plugin(&EchoPlugin, request, &policy()),
            Err(PluginError::PluginIdMismatch {
                requested: "other".into(),
                actual: "echo".into(),
            })
        );
    }

    #[test]
    fn adapts_success_to_execution_result() {
        let started_at = Utc::now();
        let response = PluginResponse {
            output: json!({"safe": true}),
            findings: vec![],
            artifacts: vec![PluginArtifact {
                name: "report.json".into(),
                content: b"{}".to_vec(),
            }],
            diagnostics: vec!["complete".into()],
            usage: PluginUsage {
                input_bytes: 2,
                output_bytes: 13,
                findings: 0,
                artifacts: 1,
                artifact_bytes: 2,
                elapsed_ms: 1,
            },
        };
        let result =
            plugin_result_to_execution_result("action-1", "echo", started_at, Ok(response));
        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(result.artifacts, vec!["report.json"]);
        assert_eq!(result.verification.as_ref().unwrap()["plugin_id"], "echo");
        assert!(result.error.is_none());
    }

    #[test]
    fn adapts_failure_to_execution_result() {
        let result = plugin_result_to_execution_result(
            "action-2",
            "echo",
            Utc::now(),
            Err(PluginError::CapabilityDenied("source.read".into())),
        );
        assert_eq!(result.status, ExecutionStatus::Failed);
        assert_eq!(result.action_id, "action-2");
        assert!(result.stderr.contains("source.read"));
        assert!(result.error.is_some());
        assert!(result.artifacts.is_empty());
    }
}
