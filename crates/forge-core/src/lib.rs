//! ForgeCore is the trusted execution boundary for AutoDev.
//!
//! Agents produce intent. ForgeCore executes only intent that has passed
//! policy evaluation and workspace confinement. The kernel also provides
//! deterministic repository exploration, skill routing, dispatch planning, and
//! model assignment primitives that can prepare bounded execution plans.

pub mod action;
pub mod agent;
pub mod context;
pub mod development_loop;
pub mod dispatch;
pub mod envelope;
pub mod error;
pub mod evidence;
pub mod execute;
pub mod git;
pub mod model;
pub mod model_assignment;
pub mod orchestrator;
pub mod patch;
pub mod patch_exec;
pub mod plugin;
pub mod policy;
pub mod read;
pub mod runtime;
pub mod skill;
pub mod verification;
pub mod workspace;
pub mod write;

pub use action::{ActionType, AgentAction, Capability, RiskLevel};
pub use agent::{
    default_profiles, AgentCapability, AgentError, AgentHealth, AgentInstance, AgentPolicy,
    AgentProfile, AgentRegistry, AgentRole, AgentState, ModelRequirement, RetryPolicy,
};
pub use context::{select_context, ContextFile, ContextItem, ContextPack, ContextPolicy};
pub use development_loop::{
    DevelopmentLoop, DevelopmentLoopError, DevelopmentLoopOutcome, DevelopmentLoopResult,
};
pub use dispatch::{plan_dispatch, DispatchPlan, SkillAssignment, UnassignedSkill};
pub use envelope::{
    ContextRefs, EnvelopeError, EnvelopeState, EvidenceBinding, ExecutionEnvelope, Lifecycle,
    PolicyBinding,
};
pub use error::{ExecutionError, ExecutionErrorKind};
pub use evidence::{
    action_id_from_record, action_type_from_record, record_from, Artifact, ArtifactHash,
    ArtifactHashAlgo, Evidence, EvidenceStore, ExecutionErrorInfo, ExecutionRecord,
    ExecutionResult, ExecutionStatus, PolicyOutcome, ReadMetadata,
};
pub use execute::execute_process;
pub use git::{execute_git, BranchInfo, Checkpoint, GitDiff, GitStatus, GitTier, RepositoryInfo};
pub use model::{
    route, Message, MockProvider, Model, ModelCapabilities, ModelError, ModelHealth, ModelOptions,
    ModelProvider, ModelRequest, ModelResponse, OllamaProvider, RouteCandidate, RoutingFactor,
    RoutingPolicy, Usage,
};
pub use model_assignment::{
    resolve_models, AvailableModel, ModelAssignment, ModelPlan, UnresolvedModel,
};
pub use orchestrator::{
    default_repairer, default_verifier, Assigner, Checkpointer, Decomposer, ExecResult,
    Orchestrator, OrchestratorError, Phase, Planner, Repairer, TaskExecutor, TaskGraph, TaskNode,
    TaskStatus, Transition, Verdict, Verifier,
};
pub use patch::{
    ApplyMode, LineKind, Patch, PatchFailure, PatchFailureReason, PatchHunk, PatchLine,
    PatchParseError, PatchResult,
};
pub use patch_exec::{patch_file, PatchMode};
pub use plugin::{
    execute_plugin, plugin_result_to_execution_result, Plugin, PluginArtifact, PluginError,
    PluginFinding, PluginLimits, PluginLocation, PluginPolicy, PluginRequest, PluginResponse,
    PluginUsage,
};
pub use policy::{evaluate_policy, has_required_capability, validate_action, PolicyDecision};
pub use read::read_file;
pub use runtime::{
    AgentRuntime, AgentRuntimeState, Executor, RuntimeError, StepOutcome, StructuredOutput, Task,
};
pub use skill::{
    default_skills, route_skills, DevelopmentContract, SkillDefinition, SkillError, SkillRegistry,
    SkillRoute, SkillRoutingEvidence,
};
pub use verification::{
    command_verifier, default_fabric, mock_verifier, verdict_from_report, Finding,
    VerificationContext, VerificationError, VerificationFabric, VerificationKind,
    VerificationReport, VerificationResult, VerificationStatus, VerificationVerdict, VerifierFn,
};
pub use workspace::{PathResolution, Workspace};
pub use write::{write_file, WriteMode};

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
/// This is the only path that can produce workspace side effects. Every action
/// is dispatched only after the typed policy/capability boundary has been
/// established by its executor module.
pub fn execute(exec: &ExecutableAction) -> Result<ExecutionResult, ExecutionError> {
    let mut result = match exec.action.action_type {
        ActionType::ReadFile => read::read_file(&exec.action, &exec.workspace)?,
        ActionType::WriteFile => {
            write::write_file(&exec.action, &exec.workspace, WriteMode::Atomic)?
        }
        ActionType::PatchFile => {
            patch_exec::patch_file(&exec.action, &exec.workspace, PatchMode::Apply)?
        }
        ActionType::Execute => execute::execute_process(&exec.action, &exec.workspace)?,
        ActionType::Git => git::execute_git(&exec.action, &exec.workspace)?,
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
