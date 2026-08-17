//! Evidence-gated capability-gap discovery and candidate staging.
//!
//! This module is deliberately side-effect free. It classifies observed
//! development gaps, renders reviewable candidate artifacts, and emits ordinary
//! [`AgentAction`] write proposals. It never executes those proposals, grants
//! capabilities, changes policy, or promotes a candidate into production.

use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{ActionType, AgentAction, Capability, RiskLevel};

/// The mechanism-level shape of an observed development weakness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapKind {
    /// Reusable judgment-dependent procedure that belongs in a skill.
    ReusableProcedure,
    /// Missing external service or data capability that may justify MCP.
    ExternalCapability,
    /// A must-run invariant that belongs in a deterministic hook.
    DeterministicGuard,
    /// A privileged operation that must remain behind ForgeCore authority.
    PrivilegedExecution,
    /// Model/agent selection or routing weakness.
    Routing,
    /// Repository-context or retrieval weakness.
    Context,
    /// Measured runtime or resource bottleneck.
    Performance,
    /// Client-side workflow or interaction weakness.
    Workflow,
}

/// Candidate mechanism selected for a gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateKind {
    Skill,
    McpServer,
    Hook,
    ForgeCoreAdapter,
    RoutingPolicy,
    ContextStrategy,
    OptimizationExperiment,
    ClientFeature,
}

/// An evidence-backed observation that may justify a capability experiment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GapObservation {
    pub id: String,
    pub objective_id: String,
    pub kind: GapKind,
    pub summary: String,
    pub evidence_refs: Vec<String>,
    pub frequency: u32,
    pub severity: u8,
    pub confidence: u8,
}

/// One file produced by a candidate generator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateArtifact {
    pub path: String,
    pub content: String,
}

/// A bounded, reviewable response to one observed gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityCandidate {
    pub id: String,
    pub objective_id: String,
    pub source_gap_id: String,
    pub kind: CandidateKind,
    pub rationale: String,
    pub evidence_refs: Vec<String>,
    pub artifacts: Vec<CandidateArtifact>,
}

/// Baseline-versus-candidate behavioral evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateEvaluation {
    pub candidate_id: String,
    /// Success rate in basis points, where 10_000 means 100%.
    pub baseline_success_bps: u16,
    /// Success rate in basis points, where 10_000 means 100%.
    pub candidate_success_bps: u16,
    /// Count of security or authority regressions observed during evaluation.
    pub safety_regressions: u32,
    pub evidence_refs: Vec<String>,
}

/// Promotion recommendation. Promotion is advisory and never mutates policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionDecision {
    Promote,
    RejectMissingEvidence,
    RejectNoImprovement,
    RejectSafetyRegression,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CapabilityGapError {
    #[error("field `{0}` must not be empty")]
    EmptyField(&'static str),
    #[error("candidate id `{0}` is not a safe lowercase slug")]
    InvalidCandidateId(String),
    #[error("gap `{0}` must reference at least one evidence record")]
    MissingEvidence(String),
    #[error("gap `{id}` has invalid {field} value {value}; expected 0..=100")]
    InvalidPercentage {
        id: String,
        field: &'static str,
        value: u8,
    },
    #[error("artifact path `{0}` is outside an allowed candidate namespace")]
    UnsafeArtifactPath(String),
}

/// Deterministically classify evidence-backed gaps into candidate mechanisms.
pub fn discover_candidates(
    observations: &[GapObservation],
) -> Result<Vec<CapabilityCandidate>, CapabilityGapError> {
    let mut ordered: Vec<&GapObservation> = observations.iter().collect();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));

    ordered.into_iter().map(candidate_from_gap).collect()
}

/// Convert generated files into untrusted write proposals.
///
/// The returned actions still require the normal ForgeCore policy,
/// authorization, workspace-confinement, and evidence path before anything is
/// persisted.
pub fn propose_candidate_writes(
    candidate: &CapabilityCandidate,
    task_id: &str,
    agent_id: &str,
) -> Result<Vec<AgentAction>, CapabilityGapError> {
    required(task_id, "task_id")?;
    required(agent_id, "agent_id")?;
    validate_candidate_id(&candidate.id)?;

    candidate
        .artifacts
        .iter()
        .enumerate()
        .map(|(index, artifact)| {
            validate_artifact_path(&artifact.path)?;
            Ok(AgentAction {
                id: format!("candidate-{}-{index}", candidate.id),
                task_id: task_id.to_string(),
                agent_id: agent_id.to_string(),
                action_type: ActionType::WriteFile,
                reason: format!(
                    "Stage evidence-gated candidate {} for independent review",
                    candidate.id
                ),
                risk: RiskLevel::Medium,
                capabilities: vec![Capability::WriteFile],
                payload: json!({
                    "operation": "write_file",
                    "path": artifact.path,
                    "content": artifact.content,
                }),
                expected: json!({
                    "artifact_path": artifact.path,
                    "status": "candidate_only",
                }),
            })
        })
        .collect()
}

/// Evaluate whether a candidate has earned a promotion recommendation.
///
/// Safety regressions are an unconditional rejection. A candidate must also
/// have evidence and strictly improve its target behavioral success rate.
pub fn evaluate_candidate(evaluation: &CandidateEvaluation) -> PromotionDecision {
    if evaluation.safety_regressions > 0 {
        return PromotionDecision::RejectSafetyRegression;
    }
    if evaluation.evidence_refs.is_empty() {
        return PromotionDecision::RejectMissingEvidence;
    }
    if evaluation.candidate_success_bps <= evaluation.baseline_success_bps {
        return PromotionDecision::RejectNoImprovement;
    }
    PromotionDecision::Promote
}

fn candidate_from_gap(gap: &GapObservation) -> Result<CapabilityCandidate, CapabilityGapError> {
    validate_gap(gap)?;
    let kind = candidate_kind(gap.kind);
    let artifacts = vec![candidate_artifact(gap, kind)?];

    Ok(CapabilityCandidate {
        id: gap.id.clone(),
        objective_id: gap.objective_id.clone(),
        source_gap_id: gap.id.clone(),
        kind,
        rationale: rationale_for(kind).to_string(),
        evidence_refs: gap.evidence_refs.clone(),
        artifacts,
    })
}

fn validate_gap(gap: &GapObservation) -> Result<(), CapabilityGapError> {
    validate_candidate_id(&gap.id)?;
    required(&gap.objective_id, "objective_id")?;
    required(&gap.summary, "summary")?;
    if gap.evidence_refs.is_empty()
        || gap.evidence_refs.iter().any(|reference| reference.trim().is_empty())
    {
        return Err(CapabilityGapError::MissingEvidence(gap.id.clone()));
    }
    for (field, value) in [("severity", gap.severity), ("confidence", gap.confidence)] {
        if value > 100 {
            return Err(CapabilityGapError::InvalidPercentage {
                id: gap.id.clone(),
                field,
                value,
            });
        }
    }
    Ok(())
}

fn candidate_kind(kind: GapKind) -> CandidateKind {
    match kind {
        GapKind::ReusableProcedure => CandidateKind::Skill,
        GapKind::ExternalCapability => CandidateKind::McpServer,
        GapKind::DeterministicGuard => CandidateKind::Hook,
        GapKind::PrivilegedExecution => CandidateKind::ForgeCoreAdapter,
        GapKind::Routing => CandidateKind::RoutingPolicy,
        GapKind::Context => CandidateKind::ContextStrategy,
        GapKind::Performance => CandidateKind::OptimizationExperiment,
        GapKind::Workflow => CandidateKind::ClientFeature,
    }
}

fn candidate_artifact(
    gap: &GapObservation,
    kind: CandidateKind,
) -> Result<CandidateArtifact, CapabilityGapError> {
    let artifact = match kind {
        CandidateKind::Skill => CandidateArtifact {
            path: format!(".cline/skills/{}/SKILL.md", gap.id),
            content: render_skill_candidate(gap),
        },
        CandidateKind::McpServer => CandidateArtifact {
            path: format!(".cline/mcp/generated/{}.json", gap.id),
            content: render_mcp_candidate(gap),
        },
        CandidateKind::Hook => CandidateArtifact {
            path: format!(".cline/hooks/generated/{}.json", gap.id),
            content: render_generic_candidate(gap, kind),
        },
        _ => CandidateArtifact {
            path: format!("docs/autodev/candidates/{}.json", gap.id),
            content: render_generic_candidate(gap, kind),
        },
    };
    validate_artifact_path(&artifact.path)?;
    Ok(artifact)
}

fn render_skill_candidate(gap: &GapObservation) -> String {
    format!(
        "---\nname: {}\ndescription: Applies the evaluated procedure candidate for capability gap {}. Use only while reproducing and measuring this specific gap.\n---\n\n# Candidate procedure\n\n1. Reproduce capability gap `{}` from its recorded evidence.\n2. Apply the smallest procedure that addresses the observed failure.\n3. Run the objective's existing verification checks.\n4. Compare behavioral success against the recorded baseline.\n5. Reject this candidate if it does not improve the target metric or if any safety regression appears.\n\nThis file is a candidate artifact. It grants no capability and does not override runtime policy.\n",
        gap.id, gap.id, gap.id
    )
}

fn render_mcp_candidate(gap: &GapObservation) -> String {
    serde_json::to_string_pretty(&json!({
        "id": gap.id,
        "kind": "mcp_server_candidate",
        "source_gap_id": gap.id,
        "objective_id": gap.objective_id,
        "status": "candidate",
        "authority": "proposal_only",
        "evidence_refs": gap.evidence_refs,
        "activation": "disabled_until_evaluated",
        "note": "Research and verify a concrete MCP implementation before adding it to an active profile. No command, URL, credential, or capability grant is synthesized here."
    }))
    .expect("candidate MCP metadata is JSON serializable")
}

fn render_generic_candidate(gap: &GapObservation, kind: CandidateKind) -> String {
    serde_json::to_string_pretty(&json!({
        "id": gap.id,
        "kind": kind,
        "source_gap_id": gap.id,
        "objective_id": gap.objective_id,
        "status": "candidate",
        "evidence_refs": gap.evidence_refs,
        "activation": "disabled_until_evaluated"
    }))
    .expect("candidate metadata is JSON serializable")
}

fn rationale_for(kind: CandidateKind) -> &'static str {
    match kind {
        CandidateKind::Skill => "A judgment-dependent reusable procedure belongs in an evaluated skill.",
        CandidateKind::McpServer => {
            "An external service or data capability belongs behind a separately evaluated MCP adapter."
        }
        CandidateKind::Hook => "A must-run invariant belongs in a deterministic runtime hook.",
        CandidateKind::ForgeCoreAdapter => {
            "A privileged capability must remain behind ForgeCore authorization and execution."
        }
        CandidateKind::RoutingPolicy => "A routing weakness should be tested as a policy candidate.",
        CandidateKind::ContextStrategy => {
            "A retrieval weakness should be tested as a bounded context strategy."
        }
        CandidateKind::OptimizationExperiment => {
            "A measured performance bottleneck should be addressed by a benchmarked experiment."
        }
        CandidateKind::ClientFeature => {
            "A workflow weakness should be addressed at the client boundary without adding execution authority."
        }
    }
}

fn validate_candidate_id(id: &str) -> Result<(), CapabilityGapError> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && !id.starts_with('-')
        && !id.ends_with('-')
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if valid {
        Ok(())
    } else {
        Err(CapabilityGapError::InvalidCandidateId(id.to_string()))
    }
}

fn validate_artifact_path(path: &str) -> Result<(), CapabilityGapError> {
    let allowed_root = path.starts_with(".cline/skills/")
        || path.starts_with(".cline/mcp/generated/")
        || path.starts_with(".cline/hooks/generated/")
        || path.starts_with("docs/autodev/candidates/");
    let safe_segments = !path.starts_with('/')
        && !path.contains('\\')
        && path.split('/').all(|segment| segment != "..");
    if allowed_root && safe_segments {
        Ok(())
    } else {
        Err(CapabilityGapError::UnsafeArtifactPath(path.to_string()))
    }
}

fn required(value: &str, field: &'static str) -> Result<(), CapabilityGapError> {
    if value.trim().is_empty() {
        Err(CapabilityGapError::EmptyField(field))
    } else {
        Ok(())
    }
}
