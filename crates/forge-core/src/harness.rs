//! Declarative harness profiles for AutoDev development workflows.
//!
//! Harness configuration expresses intent, orchestration stages, asset
//! references, and verification contracts. It is deliberately authority-free:
//! profiles cannot authorize or execute effects and cannot mark their own work
//! verified. ForgeCore policy and execution remain the trusted boundary.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Broad family of a development harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessKind {
    Sdlc,
    Agile,
    Innovation,
    Optimizer,
    Meta,
}

/// Normalized kind for an asset referenced by a harness profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessAssetKind {
    Skill,
    AgentProfile,
    Tool,
    McpServer,
    Hook,
    Prompt,
    Policy,
    Workflow,
    Evaluator,
    ContextProvider,
}

/// Versioned reference to a declarative harness asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessAssetRef {
    pub id: String,
    pub version: String,
    pub kind: HarnessAssetKind,
    pub required: bool,
}

/// One ordered stage in a harness profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessStage {
    pub id: String,
    pub objective: String,
    #[serde(default)]
    pub assets: Vec<HarnessAssetRef>,
    #[serde(default)]
    pub verification: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_group: Option<String>,
    #[serde(default)]
    pub approval_gate: bool,
}

/// A versioned, authority-free development harness profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessProfile {
    pub id: String,
    pub version: String,
    pub name: String,
    pub kind: HarnessKind,
    pub objective: String,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub stages: Vec<HarnessStage>,
    #[serde(default)]
    pub success_metrics: Vec<String>,
    #[serde(default)]
    pub memory_policy: Vec<String>,
    #[serde(default)]
    pub improvement_policy: Vec<String>,
}

/// Validation and registration errors for harness configuration.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HarnessError {
    #[error("harness profile '{profile_id}' has empty required field '{field}'")]
    EmptyField {
        profile_id: String,
        field: &'static str,
    },
    #[error("harness profile '{profile_id}' contains duplicate stage '{stage_id}'")]
    DuplicateStage {
        profile_id: String,
        stage_id: String,
    },
    #[error("harness profile '{profile_id}' stage '{stage_id}' has no verification contract")]
    MissingVerification {
        profile_id: String,
        stage_id: String,
    },
    #[error("harness profile '{profile_id}' stage '{stage_id}' has invalid asset: {reason}")]
    InvalidAsset {
        profile_id: String,
        stage_id: String,
        reason: String,
    },
    #[error("harness profile '{0}' is already registered")]
    DuplicateProfile(String),
}

impl HarnessProfile {
    /// Validate structural invariants without executing or authorizing effects.
    pub fn validate(&self) -> Result<(), HarnessError> {
        self.require_non_empty("id", &self.id)?;
        self.require_non_empty("version", &self.version)?;
        self.require_non_empty("name", &self.name)?;
        self.require_non_empty("objective", &self.objective)?;
        if self.triggers.is_empty() || self.triggers.iter().any(|term| term.trim().is_empty()) {
            return Err(HarnessError::EmptyField {
                profile_id: self.id.clone(),
                field: "triggers",
            });
        }
        if self.stages.is_empty() {
            return Err(HarnessError::EmptyField {
                profile_id: self.id.clone(),
                field: "stages",
            });
        }

        let mut stage_ids = BTreeSet::new();
        for stage in &self.stages {
            if stage.id.trim().is_empty() {
                return Err(HarnessError::EmptyField {
                    profile_id: self.id.clone(),
                    field: "stage.id",
                });
            }
            if !stage_ids.insert(stage.id.as_str()) {
                return Err(HarnessError::DuplicateStage {
                    profile_id: self.id.clone(),
                    stage_id: stage.id.clone(),
                });
            }
            if stage.objective.trim().is_empty() {
                return Err(HarnessError::EmptyField {
                    profile_id: self.id.clone(),
                    field: "stage.objective",
                });
            }
            if stage.verification.is_empty()
                || stage
                    .verification
                    .iter()
                    .any(|contract| contract.trim().is_empty())
            {
                return Err(HarnessError::MissingVerification {
                    profile_id: self.id.clone(),
                    stage_id: stage.id.clone(),
                });
            }
            for asset in &stage.assets {
                if asset.id.trim().is_empty() {
                    return Err(HarnessError::InvalidAsset {
                        profile_id: self.id.clone(),
                        stage_id: stage.id.clone(),
                        reason: "asset id must not be empty".to_string(),
                    });
                }
                if asset.version.trim().is_empty() {
                    return Err(HarnessError::InvalidAsset {
                        profile_id: self.id.clone(),
                        stage_id: stage.id.clone(),
                        reason: "asset version must not be empty".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn require_non_empty(&self, field: &'static str, value: &str) -> Result<(), HarnessError> {
        if value.trim().is_empty() {
            Err(HarnessError::EmptyField {
                profile_id: self.id.clone(),
                field,
            })
        } else {
            Ok(())
        }
    }
}

/// Deterministic registry of validated harness profiles.
#[derive(Debug, Clone, Default)]
pub struct HarnessRegistry {
    profiles: Vec<HarnessProfile>,
}

impl HarnessRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, profile: HarnessProfile) -> Result<(), HarnessError> {
        profile.validate()?;
        if self
            .profiles
            .iter()
            .any(|candidate| candidate.id == profile.id)
        {
            return Err(HarnessError::DuplicateProfile(profile.id));
        }
        self.profiles.push(profile);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&HarnessProfile> {
        self.profiles.iter().find(|profile| profile.id == id)
    }

    pub fn profiles(&self) -> &[HarnessProfile] {
        &self.profiles
    }
}

fn builtin_asset(id: &str, kind: HarnessAssetKind) -> HarnessAssetRef {
    HarnessAssetRef {
        id: id.to_string(),
        version: "builtin-v1".to_string(),
        kind,
        required: true,
    }
}

fn builtin_stage(
    id: &str,
    objective: &str,
    assets: Vec<HarnessAssetRef>,
    verification: &str,
    parallel_group: Option<&str>,
    approval_gate: bool,
) -> HarnessStage {
    HarnessStage {
        id: id.to_string(),
        objective: objective.to_string(),
        assets,
        verification: vec![verification.to_string()],
        parallel_group: parallel_group.map(str::to_string),
        approval_gate,
    }
}

fn forgeflow_sdlc() -> HarnessProfile {
    HarnessProfile {
        id: "forgeflow-sdlc".to_string(),
        version: "0.1.0".to_string(),
        name: "ForgeFlow SDLC".to_string(),
        kind: HarnessKind::Sdlc,
        objective: "move a software change from discovery through independently verified delivery"
            .to_string(),
        triggers: vec![
            "feature".to_string(),
            "implementation".to_string(),
            "bugfix".to_string(),
            "sdlc".to_string(),
            "pull request".to_string(),
        ],
        stages: vec![
            builtin_stage(
                "discover",
                "map repository structure, existing behavior, constraints, and authority boundaries",
                vec![
                    builtin_asset("github", HarnessAssetKind::Tool),
                    builtin_asset("deepwiki", HarnessAssetKind::ContextProvider),
                    builtin_asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                ],
                "repository evidence identifies the existing flow and protected boundaries",
                None,
                false,
            ),
            builtin_stage(
                "requirements",
                "convert the approved goal into explicit observable requirements",
                vec![
                    builtin_asset("requirements-extractor", HarnessAssetKind::Tool),
                    builtin_asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                ],
                "requirements are traceable to the approved goal and contain measurable acceptance criteria",
                None,
                true,
            ),
            builtin_stage(
                "architecture",
                "select a minimal architecture using repository evidence and current technical knowledge",
                vec![
                    builtin_asset("context7", HarnessAssetKind::ContextProvider),
                    builtin_asset("alphaxiv", HarnessAssetKind::ContextProvider),
                    builtin_asset("parallel-search", HarnessAssetKind::Tool),
                ],
                "architecture alternatives and trade-offs are recorded before implementation",
                None,
                true,
            ),
            builtin_stage(
                "plan",
                "decompose the approved architecture into dependency-aware implementation tasks",
                vec![builtin_asset(
                    "superpowers:writing-plans",
                    HarnessAssetKind::Skill,
                )],
                "each task names files, tests, interfaces, and verification commands",
                None,
                false,
            ),
            builtin_stage(
                "isolated-implementation",
                "implement each task in an isolated workspace using RED-GREEN-REFACTOR",
                vec![
                    builtin_asset(
                        "superpowers:using-git-worktrees",
                        HarnessAssetKind::Skill,
                    ),
                    builtin_asset(
                        "superpowers:test-driven-development",
                        HarnessAssetKind::Skill,
                    ),
                ],
                "a failing acceptance test is observed before production behavior and passes afterward",
                None,
                false,
            ),
            builtin_stage(
                "review",
                "subject the change to independent code and requirements review",
                vec![
                    builtin_asset("coderabbit", HarnessAssetKind::Tool),
                    builtin_asset(
                        "superpowers:requesting-code-review",
                        HarnessAssetKind::Skill,
                    ),
                ],
                "load-bearing review findings are resolved or explicitly rejected with evidence",
                "review",
                false,
            ),
            builtin_stage(
                "verify",
                "run canonical repository gates and compare evidence to acceptance criteria",
                vec![
                    builtin_asset("github-actions", HarnessAssetKind::Tool),
                    builtin_asset(
                        "superpowers:verification-before-completion",
                        HarnessAssetKind::Skill,
                    ),
                ],
                "canonical CI and focused acceptance tests pass at the exact candidate revision",
                None,
                false,
            ),
            builtin_stage(
                "retrospective",
                "capture evidence-backed lessons and preserve branch outcome",
                vec![
                    builtin_asset("engram", HarnessAssetKind::Tool),
                    builtin_asset(
                        "superpowers:finishing-a-development-branch",
                        HarnessAssetKind::Skill,
                    ),
                ],
                "retrospective distinguishes observed outcomes from hypotheses and proposed follow-up work",
                None,
                true,
            ),
        ],
        success_metrics: vec![
            "acceptance criteria satisfied".to_string(),
            "canonical CI green".to_string(),
            "independent review resolved".to_string(),
            "authority boundary preserved".to_string(),
        ],
        memory_policy: vec![
            "record only evidence-backed outcomes, regressions, and reusable failure patterns"
                .to_string(),
        ],
        improvement_policy: vec![
            "change the workflow only after measured comparison against the current profile"
                .to_string(),
        ],
    }
}

fn sprintmesh_agile() -> HarnessProfile {
    HarnessProfile {
        id: "sprintmesh-agile".to_string(),
        version: "0.1.0".to_string(),
        name: "SprintMesh Agile".to_string(),
        kind: HarnessKind::Agile,
        objective: "adapt backlog flow and parallel engineering work to changing product conditions"
            .to_string(),
        triggers: vec![
            "sprint".to_string(),
            "backlog".to_string(),
            "agile".to_string(),
            "kanban".to_string(),
            "story".to_string(),
        ],
        stages: vec![
            builtin_stage(
                "product-goal",
                "bind work to a concrete product outcome and measurable success condition",
                vec![
                    builtin_asset("linear", HarnessAssetKind::Tool),
                    builtin_asset("requirements-extractor", HarnessAssetKind::Tool),
                ],
                "the product goal has an owner, observable outcome, and explicit constraints",
                None,
                true,
            ),
            builtin_stage(
                "backlog-refinement",
                "decompose work into independently valuable, testable backlog items",
                vec![
                    builtin_asset("linear", HarnessAssetKind::Tool),
                    builtin_asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                ],
                "stories have acceptance criteria, dependencies, and bounded scope",
                None,
                false,
            ),
            builtin_stage(
                "prioritization",
                "rank backlog work using impact, confidence, effort, risk, and dependency pressure",
                vec![builtin_asset("rice-plus-prioritizer", HarnessAssetKind::Prompt)],
                "priority ordering includes the evidence and assumptions behind each score",
                None,
                false,
            ),
            builtin_stage(
                "flow-selection",
                "choose Scrum, Kanban, Scrumban, spike, or single-feature flow from observed volatility and coupling",
                vec![builtin_asset("adaptive-agile-selector", HarnessAssetKind::Prompt)],
                "selected flow is justified by requirement volatility, dependency coupling, uncertainty, and risk",
                None,
                false,
            ),
            builtin_stage(
                "parallel-execution",
                "dispatch independent backlog items concurrently while preserving ownership boundaries",
                vec![
                    builtin_asset(
                        "superpowers:dispatching-parallel-agents",
                        HarnessAssetKind::Skill,
                    ),
                    builtin_asset("github", HarnessAssetKind::Tool),
                ],
                "parallel work has disjoint ownership or an explicit integration contract",
                "delivery",
                false,
            ),
            builtin_stage(
                "integration",
                "integrate completed work through independent review and repository gates",
                vec![
                    builtin_asset("coderabbit", HarnessAssetKind::Tool),
                    builtin_asset(
                        "superpowers:requesting-code-review",
                        HarnessAssetKind::Skill,
                    ),
                ],
                "integrated changes satisfy story acceptance criteria and branch protection gates",
                None,
                false,
            ),
            builtin_stage(
                "demo-retrospective",
                "compare delivered value and process behavior to the product goal, then update the backlog",
                vec![
                    builtin_asset("linear", HarnessAssetKind::Tool),
                    builtin_asset("engram", HarnessAssetKind::Tool),
                ],
                "retrospective records cycle-time, blockers, defects, and validated process changes",
                None,
                true,
            ),
        ],
        success_metrics: vec![
            "valuable backlog throughput".to_string(),
            "bounded work in progress".to_string(),
            "cycle-time trend".to_string(),
            "escaped-defect trend".to_string(),
        ],
        memory_policy: vec![
            "retain flow decisions with the volatility, coupling, risk, and throughput evidence that motivated them"
                .to_string(),
        ],
        improvement_policy: vec![
            "adapt methodology only when observed delivery metrics justify the change".to_string(),
        ],
    }
}

fn idea_tournament() -> HarnessProfile {
    HarnessProfile {
        id: "idea-tournament".to_string(),
        version: "0.1.0".to_string(),
        name: "IdeaTournament".to_string(),
        kind: HarnessKind::Innovation,
        objective: "generate many materially different ideas and select evidence-backed, strategically robust experiments"
            .to_string(),
        triggers: vec![
            "idea".to_string(),
            "brainstorm".to_string(),
            "triz".to_string(),
            "six thinking hats".to_string(),
            "game theory".to_string(),
            "rice".to_string(),
        ],
        stages: vec![
            builtin_stage(
                "problem-model",
                "model desired outcome, current state, constraints, resources, unknowns, and failure modes",
                vec![
                    builtin_asset("requirements-extractor", HarnessAssetKind::Tool),
                    builtin_asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                ],
                "the problem model separates evidence, assumptions, constraints, and desired outcomes",
                None,
                false,
            ),
            builtin_stage(
                "autotriz",
                "identify contradictions and generate solutions that restructure rather than merely compromise them",
                vec![builtin_asset(
                    "autotriz-contradiction-resolver",
                    HarnessAssetKind::Prompt,
                )],
                "each retained concept names the contradiction it resolves and the mechanism of resolution",
                "ideation",
                false,
            ),
            builtin_stage(
                "six-hats",
                "evaluate the problem independently through evidence, intuition, risk, upside, creativity, and orchestration lenses",
                vec![builtin_asset("six-thinking-hats", HarnessAssetKind::Prompt)],
                "all six perspectives are represented without collapsing minority risks or opportunities",
                "ideation",
                false,
            ),
            builtin_stage(
                "research-validation",
                "test novelty and feasibility against current implementations, documentation, and academic evidence",
                vec![
                    builtin_asset("parallel-search", HarnessAssetKind::Tool),
                    builtin_asset("alphaxiv", HarnessAssetKind::ContextProvider),
                    builtin_asset("context7", HarnessAssetKind::ContextProvider),
                    builtin_asset("github", HarnessAssetKind::Tool),
                ],
                "claims of novelty and feasibility are linked to independent evidence or explicitly marked uncertain",
                "research",
                false,
            ),
            builtin_stage(
                "rice-plus",
                "score candidates using RICE plus novelty, reversibility, information gain, optionality, and technical leverage",
                vec![builtin_asset("rice-plus-evaluator", HarnessAssetKind::Evaluator)],
                "scores expose component values and confidence rather than only a final rank",
                None,
                false,
            ),
            builtin_stage(
                "game-theory",
                "stress-test candidates against strategic responses from users, maintainers, competitors, attackers, and platform vendors",
                vec![builtin_asset("game-theory-stress-test", HarnessAssetKind::Evaluator)],
                "fragile concepts that require unrealistic cooperation are identified and penalized",
                None,
                false,
            ),
            builtin_stage(
                "prototype-selection",
                "select the smallest reversible experiments that maximize information gain and strategic value",
                vec![
                    builtin_asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                    builtin_asset("experiment-selection", HarnessAssetKind::Evaluator),
                ],
                "selected experiments have falsifiable hypotheses, success metrics, and rollback paths",
                None,
                true,
            ),
        ],
        success_metrics: vec![
            "concept diversity".to_string(),
            "evidence coverage".to_string(),
            "contradiction resolution quality".to_string(),
            "information gain per experiment".to_string(),
        ],
        memory_policy: vec![
            "retain rejected concepts with rejection evidence so future ideation does not repeat disproven paths"
                .to_string(),
        ],
        improvement_policy: vec![
            "adjust scoring weights only from retrospective experiment outcomes, never from preference alone"
                .to_string(),
        ],
    }
}

fn optiforge_optimizer() -> HarnessProfile {
    HarnessProfile {
        id: "optiforge-optimizer".to_string(),
        version: "0.1.0".to_string(),
        name: "OptiForge Optimizer".to_string(),
        kind: HarnessKind::Optimizer,
        objective: "improve software-development throughput or quality through measured, reversible experiments"
            .to_string(),
        triggers: vec![
            "optimize".to_string(),
            "performance".to_string(),
            "bottleneck".to_string(),
            "build time".to_string(),
            "ci time".to_string(),
            "developer productivity".to_string(),
        ],
        stages: vec![
            builtin_stage(
                "observe",
                "collect repository, workflow, quality, latency, cost, and failure signals without changing behavior",
                vec![
                    builtin_asset("github", HarnessAssetKind::Tool),
                    builtin_asset("workflow-generator", HarnessAssetKind::McpServer),
                ],
                "observations are timestamped and attributable to reproducible sources",
                None,
                false,
            ),
            builtin_stage(
                "baseline",
                "establish pre-change metrics and a regression budget",
                vec![builtin_asset("benchmark-baseline", HarnessAssetKind::Evaluator)],
                "baseline contains sample size, metric definitions, and repeatable measurement procedure",
                None,
                false,
            ),
            builtin_stage(
                "bottleneck-detection",
                "rank constraints by measured impact on delivery throughput, correctness, or resource use",
                vec![builtin_asset("workflow-generator", HarnessAssetKind::McpServer)],
                "the selected bottleneck is supported by measured evidence rather than code aesthetics",
                None,
                false,
            ),
            builtin_stage(
                "hypothesis",
                "generate minimally invasive interventions using current documentation, implementations, and research",
                vec![
                    builtin_asset("context7", HarnessAssetKind::ContextProvider),
                    builtin_asset("parallel-search", HarnessAssetKind::Tool),
                    builtin_asset("alphaxiv", HarnessAssetKind::ContextProvider),
                ],
                "each intervention states a falsifiable expected metric change and possible regressions",
                "research",
                false,
            ),
            builtin_stage(
                "controlled-experiment",
                "apply one reversible intervention in isolation with regression tests",
                vec![
                    builtin_asset(
                        "superpowers:using-git-worktrees",
                        HarnessAssetKind::Skill,
                    ),
                    builtin_asset(
                        "superpowers:test-driven-development",
                        HarnessAssetKind::Skill,
                    ),
                ],
                "experiment differs from baseline only by the intended intervention and required test instrumentation",
                None,
                false,
            ),
            builtin_stage(
                "benchmark",
                "repeat baseline measurements and quantify benefit, variance, and regressions",
                vec![builtin_asset("benchmark-comparison", HarnessAssetKind::Evaluator)],
                "before/after measurements use the same procedure and expose sample sizes",
                None,
                false,
            ),
            builtin_stage(
                "decision",
                "keep, reject, or investigate the intervention from measured outcomes",
                vec![builtin_asset(
                    "superpowers:verification-before-completion",
                    HarnessAssetKind::Skill,
                )],
                "accepted changes improve at least one target metric without breaching correctness or safety budgets",
                None,
                true,
            ),
            builtin_stage(
                "learn",
                "store successful and failed optimization hypotheses with their evidence",
                vec![builtin_asset("engram", HarnessAssetKind::Tool)],
                "learning record contains baseline, intervention, outcome, and confidence",
                None,
                false,
            ),
        ],
        success_metrics: vec![
            "target metric improvement".to_string(),
            "no protected-metric regression".to_string(),
            "experiment reproducibility".to_string(),
            "rollback readiness".to_string(),
        ],
        memory_policy: vec![
            "store baselines and rejected hypotheses to make future optimization cumulative"
                .to_string(),
        ],
        improvement_policy: vec![
            "retain optimizer changes only when repeated measurements outperform the prior baseline"
                .to_string(),
        ],
    }
}

fn harnessforge_meta() -> HarnessProfile {
    HarnessProfile {
        id: "harnessforge-meta".to_string(),
        version: "0.1.0".to_string(),
        name: "HarnessForge Meta-Harness".to_string(),
        kind: HarnessKind::Meta,
        objective: "discover recurring development work and compile it into versioned, evaluated harness candidates"
            .to_string(),
        triggers: vec![
            "harness".to_string(),
            "workflow generator".to_string(),
            "workflow".to_string(),
            "agent harness".to_string(),
            "playbook".to_string(),
        ],
        stages: vec![
            builtin_stage(
                "inventory",
                "inventory available tools, skills, MCP servers, workflows, evaluators, and policy constraints",
                vec![
                    builtin_asset("hapi-mcp-registry", HarnessAssetKind::McpServer),
                    builtin_asset("agentplaybooks", HarnessAssetKind::McpServer),
                    builtin_asset("workflows-mcp-server", HarnessAssetKind::McpServer),
                ],
                "capability inventory records provenance, version, availability, and authority level",
                None,
                false,
            ),
            builtin_stage(
                "repetition-mining",
                "identify repeated multi-step tasks, tool sequences, failure patterns, and coordination costs",
                vec![
                    builtin_asset("engram", HarnessAssetKind::Tool),
                    builtin_asset("github", HarnessAssetKind::Tool),
                ],
                "a harness candidate is supported by repeated-task evidence rather than a single anecdote",
                None,
                false,
            ),
            builtin_stage(
                "generate",
                "compose candidate harness stages from the smallest sufficient capabilities and explicit verification contracts",
                vec![
                    builtin_asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                    builtin_asset("workflows-mcp-server", HarnessAssetKind::McpServer),
                ],
                "candidate exposes inputs, stages, assets, approvals, verification, fallbacks, and success metrics",
                None,
                true,
            ),
            builtin_stage(
                "adversarial-simulation",
                "exercise candidates against ambiguous inputs, tool failures, stale evidence, conflicting agents, and unsafe requests",
                vec![
                    builtin_asset("workflow-mcp", HarnessAssetKind::McpServer),
                    builtin_asset("harness-adversary", HarnessAssetKind::Evaluator),
                ],
                "simulation includes failure-path evidence and verifies fail-closed behavior at authority boundaries",
                "evaluation",
                false,
            ),
            builtin_stage(
                "benchmark",
                "compare candidate behavior against the current harness or a generic-agent baseline",
                vec![builtin_asset("harness-benchmark", HarnessAssetKind::Evaluator)],
                "comparison uses matched tasks and reports correctness, evidence completion, safety, cost, and latency",
                "evaluation",
                false,
            ),
            builtin_stage(
                "promotion-proposal",
                "produce an advisory promotion decision from independent evaluation evidence",
                vec![builtin_asset(
                    "superpowers:verification-before-completion",
                    HarnessAssetKind::Skill,
                )],
                "promotion proposal cannot mutate registry or execution authority and cites all gating metrics",
                None,
                true,
            ),
            builtin_stage(
                "versioned-learning",
                "publish replayable candidate metadata and retain measured outcomes for future routing improvements",
                vec![
                    builtin_asset("agentplaybooks", HarnessAssetKind::McpServer),
                    builtin_asset("cool-workflow", HarnessAssetKind::Workflow),
                    builtin_asset("engram", HarnessAssetKind::Tool),
                ],
                "published metadata is versioned, replayable, provenance-bearing, and linked to its evaluation evidence",
                None,
                false,
            ),
        ],
        success_metrics: vec![
            "correctness versus baseline".to_string(),
            "evidence completion".to_string(),
            "unsafe-action rejection".to_string(),
            "workflow efficiency".to_string(),
            "replayability".to_string(),
        ],
        memory_policy: vec![
            "retain task traces, evaluation results, anti-patterns, and version relationships without storing execution authority"
                .to_string(),
        ],
        improvement_policy: vec![
            "promotion requires independent evidence that safety and correctness do not regress and at least one efficiency metric improves"
                .to_string(),
        ],
    }
}

/// Built-in v0 development harness catalog in stable deterministic order.
pub fn default_harness_profiles() -> HarnessRegistry {
    HarnessRegistry {
        profiles: vec![
            forgeflow_sdlc(),
            sprintmesh_agile(),
            idea_tournament(),
            optiforge_optimizer(),
            harnessforge_meta(),
        ],
    }
}
