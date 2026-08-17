//! ForgeCore is the trusted execution boundary for AutoDev.
//!
//! Agents produce intent. ForgeCore executes only intent that has passed
//! policy evaluation and workspace confinement. The kernel also provides
//! deterministic repository exploration, skill routing, dispatch planning, and
//! model assignment primitives that can prepare bounded execution plans.

pub mod action;
pub mod action_proposal;
pub mod agent;
pub mod codex_app_server;
pub mod codex_proposal;
pub mod codex_subscription;
pub mod context;
pub mod development_loop;
pub mod dispatch;
pub mod envelope;
pub mod error;
pub mod evidence;
pub mod execute;
mod git;
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
pub mod verified_orchestrator;
pub mod workspace;
pub mod write;

pub use action::{ActionType, AgentAction, Capability, RiskLevel};
pub use action_proposal::{propose_action, ActionProposal};
pub use agent::{
    default_profiles, AgentCapability, AgentError, AgentHealth, AgentInstance, AgentPolicy,
    AgentProfile, AgentRegistry, AgentRole, AgentState, ModelRequirement, RetryPolicy,
};
pub use codex_app_server::{CodexEventTransport, JsonlCodexTransport, StdioCodexTransport};
pub use codex_proposal::{CodexProposalClient, CodexProposalRequest};
pub use codex_subscription::{
    CodexAccount, CodexCredits, CodexLoginStart, CodexRateLimitSnapshot, CodexRateLimitWindow,
    CodexRateLimits, CodexRpcTransport, CodexServerInfo, CodexSubscriptionClient,
    CodexSubscriptionError,
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
pub use git::{BranchInfo, Checkpoint, GitDiff, GitStatus, GitTier, RepositoryInfo};
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
pub use policy::{
    enforce_policy, evaluate_policy, has_required_capability, has_required_execution_authority,
    validate_action, AuthorizationGrant, ExecutionAuthority, PolicyDecision,
};
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
pub use verified_orchestrator::{
    EnvelopeFactory, VerifiedOrchestrator, VerifiedOrchestratorError, VerifiedOrchestratorState,
};
pub use workspace::{PathResolution, Workspace};
pub use write::{write_file, WriteMode};

use chrono::Utc;

/// A validated action ready for execution, bound to a workspace and kernel-owned
/// authority. Public constructors deliberately mint deny-all capability
/// authority; trusted ForgeCore paths must bind effective authority separately.
#[derive(Debug, Clone)]
pub struct ExecutableAction {
    pub action: AgentAction,
    pub workspace: Workspace,
    pub authority: ExecutionAuthority,
    pub authorization: AuthorizationGrant,
}

impl ExecutableAction {
    /// Construct an executable action with no effective capabilities or approval.
    pub fn new(action: AgentAction, workspace: Workspace) -> Self {
        ExecutableAction {
            action,
            workspace,
            authority: ExecutionAuthority::deny_all(),
            authorization: AuthorizationGrant::none(),
        }
    }

    /// Bind an approval reference without granting execution capabilities.
    pub fn with_approval(
        action: AgentAction,
        workspace: Workspace,
        approval_ref: impl Into<String>,
    ) -> Self {
        ExecutableAction {
            action,
            workspace,
            authority: ExecutionAuthority::deny_all(),
            authorization: AuthorizationGrant::approved(approval_ref),
        }
    }

    /// Trusted ForgeCore constructor for effective capabilities without approval.
    pub(crate) fn with_authority(
        action: AgentAction,
        workspace: Workspace,
        capabilities: impl IntoIterator<Item = Capability>,
    ) -> Self {
        ExecutableAction {
            action,
            workspace,
            authority: ExecutionAuthority::from_trusted_capabilities(capabilities),
            authorization: AuthorizationGrant::none(),
        }
    }

    /// Trusted ForgeCore constructor for effective capabilities plus approval.
    pub(crate) fn with_authority_and_approval(
        action: AgentAction,
        workspace: Workspace,
        capabilities: impl IntoIterator<Item = Capability>,
        approval_ref: impl Into<String>,
    ) -> Self {
        ExecutableAction {
            action,
            workspace,
            authority: ExecutionAuthority::from_trusted_capabilities(capabilities),
            authorization: AuthorizationGrant::approved(approval_ref),
        }
    }
}

/// The top-level execution entry point.
pub fn execute(exec: &ExecutableAction) -> Result<ExecutionResult, ExecutionError> {
    let mut result = match exec.action.action_type {
        ActionType::ReadFile => {
            read::read_file_authorized(&exec.action, &exec.workspace, &exec.authorization)?
        }
        ActionType::WriteFile => write::write_file_authorized(
            &exec.action,
            &exec.workspace,
            WriteMode::Atomic,
            &exec.authority,
            &exec.authorization,
        )?,
        ActionType::PatchFile => patch_exec::patch_file_authorized(
            &exec.action,
            &exec.workspace,
            PatchMode::Apply,
            &exec.authorization,
        )?,
        ActionType::Execute => execute::execute_process(&exec.action, &exec.workspace)?,
        ActionType::Git => execute_git_authorized(exec)?,
        other => {
            return Err(ExecutionError::UnsupportedAction(
                other.as_str().to_string(),
            ))
        }
    };
    result.action_id = exec.action.id.clone();
    Ok(result)
}

/// Safe public Git entry point. Caller-supplied approval fields do not grant
/// authorization; destructive operations require an `ExecutableAction` carrying
/// a kernel-owned approval grant.
pub fn execute_git(
    action: &AgentAction,
    workspace: &Workspace,
) -> Result<ExecutionResult, ExecutionError> {
    execute(&ExecutableAction::new(action.clone(), workspace.clone()))
}

fn execute_git_authorized(exec: &ExecutableAction) -> Result<ExecutionResult, ExecutionError> {
    let mut action = exec.action.clone();
    if let Some(payload) = action.payload.as_object_mut() {
        // Never trust an approval bit supplied by an agent/model payload.
        payload.remove("approved");
        // The legacy Git adapter currently expects an internal marker. Only the
        // trusted kernel grant can recreate it after sanitization.
        if exec.authorization.is_approved() {
            payload.insert("approved".to_string(), serde_json::Value::Bool(true));
        }
    }

    let operation = action
        .payload
        .get("operation")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let capabilities = action.capabilities.clone();
    match operation {
        "repository_info" | "status" | "diff" | "branch" | "log" => {
            git::run_read(&capabilities, || git::execute_git(&action, &exec.workspace))
        }
        "checkpoint" | "prepare_commit" => {
            git::run_mutate(&capabilities, || git::execute_git(&action, &exec.workspace))
        }
        "rollback" => {
            git::run_destructive(&capabilities, || git::execute_git(&action, &exec.workspace))
        }
        _ => git::execute_git(&action, &exec.workspace),
    }
}

/// Dry-run preview: evaluates policy without touching the filesystem.
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
