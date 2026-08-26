use super::{
    HarnessAssetKind, HarnessAssetRef, HarnessKind, HarnessProfile, HarnessRegistry, HarnessStage,
};

const BUILTIN_VERSION: &str = "builtin-v1";
const PROFILE_VERSION: &str = "0.1.0";

fn asset(id: &str, kind: HarnessAssetKind) -> HarnessAssetRef {
    HarnessAssetRef {
        id: id.to_string(),
        version: BUILTIN_VERSION.to_string(),
        kind,
        required: true,
    }
}

fn stage(
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

fn profile(
    id: &str,
    name: &str,
    kind: HarnessKind,
    objective: &str,
    triggers: &[&str],
    stages: Vec<HarnessStage>,
    metrics: &[&str],
    memory_policy: &str,
    improvement_policy: &str,
) -> HarnessProfile {
    HarnessProfile {
        id: id.to_string(),
        version: PROFILE_VERSION.to_string(),
        name: name.to_string(),
        kind,
        objective: objective.to_string(),
        triggers: triggers.iter().map(|value| (*value).to_string()).collect(),
        stages,
        success_metrics: metrics.iter().map(|value| (*value).to_string()).collect(),
        memory_policy: vec![memory_policy.to_string()],
        improvement_policy: vec![improvement_policy.to_string()],
    }
}

fn forgeflow_sdlc() -> HarnessProfile {
    profile(
        "forgeflow-sdlc",
        "ForgeFlow SDLC",
        HarnessKind::Sdlc,
        "move a software change from discovery through independently verified delivery",
        &["feature", "implementation", "bugfix", "sdlc", "pull request"],
        vec![
            stage(
                "discover",
                "map repository structure, existing behavior, constraints, and authority boundaries",
                vec![
                    asset("github", HarnessAssetKind::Tool),
                    asset("deepwiki", HarnessAssetKind::ContextProvider),
                    asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                ],
                "repository evidence identifies the existing flow and protected boundaries",
                None,
                false,
            ),
            stage(
                "requirements",
                "convert the approved goal into explicit observable requirements",
                vec![
                    asset("requirements-extractor", HarnessAssetKind::Tool),
                    asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                ],
                "requirements are traceable to the approved goal and measurable acceptance criteria",
                None,
                true,
            ),
            stage(
                "architecture",
                "select a minimal architecture from repository evidence and current technical knowledge",
                vec![
                    asset("context7", HarnessAssetKind::ContextProvider),
                    asset("alphaxiv", HarnessAssetKind::ContextProvider),
                    asset("parallel-search", HarnessAssetKind::Tool),
                ],
                "alternatives and trade-offs are recorded before implementation",
                None,
                true,
            ),
            stage(
                "plan",
                "decompose the approved architecture into dependency-aware implementation tasks",
                vec![asset("superpowers:writing-plans", HarnessAssetKind::Skill)],
                "each task names files, tests, interfaces, and verification commands",
                None,
                false,
            ),
            stage(
                "isolated-implementation",
                "implement tasks in isolated workspaces using RED-GREEN-REFACTOR",
                vec![
                    asset("superpowers:using-git-worktrees", HarnessAssetKind::Skill),
                    asset("superpowers:test-driven-development", HarnessAssetKind::Skill),
                ],
                "a failing acceptance test is observed before production behavior and passes afterward",
                None,
                false,
            ),
            stage(
                "review",
                "subject the change to independent code and requirements review",
                vec![
                    asset("coderabbit", HarnessAssetKind::Tool),
                    asset("superpowers:requesting-code-review", HarnessAssetKind::Skill),
                ],
                "load-bearing findings are resolved or rejected with evidence",
                Some("review"),
                false,
            ),
            stage(
                "verify",
                "run canonical repository gates and compare evidence to acceptance criteria",
                vec![
                    asset("github-actions", HarnessAssetKind::Tool),
                    asset(
                        "superpowers:verification-before-completion",
                        HarnessAssetKind::Skill,
                    ),
                ],
                "canonical CI and focused acceptance tests pass at the exact candidate revision",
                None,
                false,
            ),
            stage(
                "retrospective",
                "capture evidence-backed lessons and preserve the branch outcome",
                vec![
                    asset("engram", HarnessAssetKind::Tool),
                    asset(
                        "superpowers:finishing-a-development-branch",
                        HarnessAssetKind::Skill,
                    ),
                ],
                "retrospective separates observed outcomes, hypotheses, and follow-up work",
                None,
                true,
            ),
        ],
        &[
            "acceptance criteria satisfied",
            "canonical CI green",
            "independent review resolved",
            "authority boundary preserved",
        ],
        "record only evidence-backed outcomes, regressions, and reusable failure patterns",
        "change the workflow only after measured comparison against the current profile",
    )
}

fn sprintmesh_agile() -> HarnessProfile {
    profile(
        "sprintmesh-agile",
        "SprintMesh Agile",
        HarnessKind::Agile,
        "adapt backlog flow and parallel engineering work to changing product conditions",
        &["sprint", "backlog", "agile", "kanban", "story"],
        vec![
            stage(
                "product-goal",
                "bind work to a measurable product outcome",
                vec![
                    asset("linear", HarnessAssetKind::Tool),
                    asset("requirements-extractor", HarnessAssetKind::Tool),
                ],
                "the product goal has an observable outcome and explicit constraints",
                None,
                true,
            ),
            stage(
                "backlog-refinement",
                "decompose work into independently valuable, testable backlog items",
                vec![
                    asset("linear", HarnessAssetKind::Tool),
                    asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                ],
                "stories have acceptance criteria, dependencies, and bounded scope",
                None,
                false,
            ),
            stage(
                "prioritization",
                "rank work using impact, confidence, effort, risk, and dependency pressure",
                vec![asset("rice-plus-prioritizer", HarnessAssetKind::Prompt)],
                "priority ordering exposes component scores, evidence, and assumptions",
                None,
                false,
            ),
            stage(
                "flow-selection",
                "choose Scrum, Kanban, Scrumban, spike, or single-feature flow from observed conditions",
                vec![asset("adaptive-agile-selector", HarnessAssetKind::Prompt)],
                "flow choice is justified by volatility, coupling, uncertainty, and risk",
                None,
                false,
            ),
            stage(
                "parallel-execution",
                "dispatch independent backlog items concurrently with explicit ownership boundaries",
                vec![
                    asset(
                        "superpowers:dispatching-parallel-agents",
                        HarnessAssetKind::Skill,
                    ),
                    asset("github", HarnessAssetKind::Tool),
                ],
                "parallel work has disjoint ownership or an explicit integration contract",
                Some("delivery"),
                false,
            ),
            stage(
                "integration",
                "integrate completed work through independent review and repository gates",
                vec![
                    asset("coderabbit", HarnessAssetKind::Tool),
                    asset("superpowers:requesting-code-review", HarnessAssetKind::Skill),
                ],
                "integrated changes satisfy story acceptance criteria and branch protection gates",
                None,
                false,
            ),
            stage(
                "demo-retrospective",
                "compare delivered value and process behavior to the goal, then adapt the backlog",
                vec![
                    asset("linear", HarnessAssetKind::Tool),
                    asset("engram", HarnessAssetKind::Tool),
                ],
                "retrospective records cycle time, blockers, defects, and validated process changes",
                None,
                true,
            ),
        ],
        &[
            "valuable backlog throughput",
            "bounded work in progress",
            "cycle-time trend",
            "escaped-defect trend",
        ],
        "retain flow decisions with the volatility, coupling, risk, and throughput evidence that motivated them",
        "adapt methodology only when observed delivery metrics justify the change",
    )
}

fn idea_tournament() -> HarnessProfile {
    profile(
        "idea-tournament",
        "IdeaTournament",
        HarnessKind::Innovation,
        "generate materially different ideas and select evidence-backed, strategically robust experiments",
        &[
            "idea",
            "brainstorm",
            "triz",
            "six thinking hats",
            "game theory",
            "rice",
        ],
        vec![
            stage(
                "problem-model",
                "model desired outcome, current state, constraints, resources, unknowns, and failure modes",
                vec![
                    asset("requirements-extractor", HarnessAssetKind::Tool),
                    asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                ],
                "the model separates evidence, assumptions, constraints, and outcomes",
                None,
                false,
            ),
            stage(
                "autotriz",
                "identify contradictions and generate solutions that restructure rather than merely compromise them",
                vec![asset(
                    "autotriz-contradiction-resolver",
                    HarnessAssetKind::Prompt,
                )],
                "each retained concept names the contradiction and resolution mechanism",
                Some("ideation"),
                false,
            ),
            stage(
                "six-hats",
                "evaluate evidence, intuition, risk, upside, creativity, and orchestration independently",
                vec![asset("six-thinking-hats", HarnessAssetKind::Prompt)],
                "all six perspectives remain visible through synthesis",
                Some("ideation"),
                false,
            ),
            stage(
                "research-validation",
                "test novelty and feasibility against current implementations, documentation, and research",
                vec![
                    asset("parallel-search", HarnessAssetKind::Tool),
                    asset("alphaxiv", HarnessAssetKind::ContextProvider),
                    asset("context7", HarnessAssetKind::ContextProvider),
                    asset("github", HarnessAssetKind::Tool),
                ],
                "novelty and feasibility claims cite evidence or remain explicitly uncertain",
                Some("research"),
                false,
            ),
            stage(
                "rice-plus",
                "score candidates using RICE plus novelty, reversibility, information gain, optionality, and leverage",
                vec![asset("rice-plus-evaluator", HarnessAssetKind::Evaluator)],
                "scores expose component values and confidence rather than only a final rank",
                None,
                false,
            ),
            stage(
                "game-theory",
                "stress-test strategic responses from users, maintainers, competitors, attackers, and platform vendors",
                vec![asset(
                    "game-theory-stress-test",
                    HarnessAssetKind::Evaluator,
                )],
                "ideas dependent on unrealistic cooperation are identified and penalized",
                None,
                false,
            ),
            stage(
                "prototype-selection",
                "select the smallest reversible experiments maximizing information gain and strategic value",
                vec![
                    asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                    asset("experiment-selection", HarnessAssetKind::Evaluator),
                ],
                "selected experiments have falsifiable hypotheses, metrics, and rollback paths",
                None,
                true,
            ),
        ],
        &[
            "concept diversity",
            "evidence coverage",
            "contradiction resolution quality",
            "information gain per experiment",
        ],
        "retain rejected concepts with rejection evidence so ideation does not repeat disproven paths",
        "adjust scoring weights only from retrospective experiment outcomes",
    )
}

fn optiforge_optimizer() -> HarnessProfile {
    profile(
        "optiforge-optimizer",
        "OptiForge Optimizer",
        HarnessKind::Optimizer,
        "improve development throughput or quality through measured, reversible experiments",
        &[
            "optimize",
            "performance",
            "bottleneck",
            "build time",
            "ci time",
            "developer productivity",
        ],
        vec![
            stage(
                "observe",
                "collect workflow, quality, latency, cost, and failure signals without changing behavior",
                vec![
                    asset("github", HarnessAssetKind::Tool),
                    asset("workflow-generator", HarnessAssetKind::McpServer),
                ],
                "observations are timestamped and attributable to reproducible sources",
                None,
                false,
            ),
            stage(
                "baseline",
                "establish pre-change metrics and a regression budget",
                vec![asset("benchmark-baseline", HarnessAssetKind::Evaluator)],
                "baseline includes sample size, metric definitions, and repeatable procedure",
                None,
                false,
            ),
            stage(
                "bottleneck-detection",
                "rank constraints by measured impact on throughput, correctness, or resources",
                vec![asset("workflow-generator", HarnessAssetKind::McpServer)],
                "selected bottleneck is supported by measurement rather than code aesthetics",
                None,
                false,
            ),
            stage(
                "hypothesis",
                "generate minimally invasive interventions using current documentation and research",
                vec![
                    asset("context7", HarnessAssetKind::ContextProvider),
                    asset("parallel-search", HarnessAssetKind::Tool),
                    asset("alphaxiv", HarnessAssetKind::ContextProvider),
                ],
                "each intervention states a falsifiable metric change and possible regressions",
                Some("research"),
                false,
            ),
            stage(
                "controlled-experiment",
                "apply one reversible intervention in isolation with regression tests",
                vec![
                    asset("superpowers:using-git-worktrees", HarnessAssetKind::Skill),
                    asset("superpowers:test-driven-development", HarnessAssetKind::Skill),
                ],
                "experiment differs from baseline only by the intended intervention and required instrumentation",
                None,
                false,
            ),
            stage(
                "benchmark",
                "repeat baseline measurements and quantify benefit, variance, and regressions",
                vec![asset("benchmark-comparison", HarnessAssetKind::Evaluator)],
                "before and after measurements use the same procedure and expose sample sizes",
                None,
                false,
            ),
            stage(
                "decision",
                "keep, reject, or investigate the intervention from measured outcomes",
                vec![asset(
                    "superpowers:verification-before-completion",
                    HarnessAssetKind::Skill,
                )],
                "accepted changes improve a target metric without breaching correctness or safety budgets",
                None,
                true,
            ),
            stage(
                "learn",
                "store successful and failed optimization hypotheses with evidence",
                vec![asset("engram", HarnessAssetKind::Tool)],
                "learning record includes baseline, intervention, outcome, and confidence",
                None,
                false,
            ),
        ],
        &[
            "target metric improvement",
            "no protected-metric regression",
            "experiment reproducibility",
            "rollback readiness",
        ],
        "store baselines and rejected hypotheses so future optimization is cumulative",
        "retain optimizer changes only when repeated measurements outperform the prior baseline",
    )
}

fn harnessforge_meta() -> HarnessProfile {
    profile(
        "harnessforge-meta",
        "HarnessForge Meta-Harness",
        HarnessKind::Meta,
        "discover recurring development work and compile it into versioned, evaluated harness candidates",
        &[
            "harness",
            "workflow generator",
            "workflow",
            "agent harness",
            "playbook",
        ],
        vec![
            stage(
                "inventory",
                "inventory tools, skills, MCP servers, workflows, evaluators, and policy constraints",
                vec![
                    asset("hapi-mcp-registry", HarnessAssetKind::McpServer),
                    asset("agentplaybooks", HarnessAssetKind::McpServer),
                    asset("workflows-mcp-server", HarnessAssetKind::McpServer),
                ],
                "inventory records provenance, version, availability, and authority level",
                None,
                false,
            ),
            stage(
                "repetition-mining",
                "identify repeated multi-step tasks, tool sequences, failure patterns, and coordination costs",
                vec![
                    asset("engram", HarnessAssetKind::Tool),
                    asset("github", HarnessAssetKind::Tool),
                ],
                "a candidate is supported by repeated-task evidence rather than a single anecdote",
                None,
                false,
            ),
            stage(
                "generate",
                "compose candidate stages from the smallest sufficient capabilities and explicit verification contracts",
                vec![
                    asset("superpowers:brainstorming", HarnessAssetKind::Skill),
                    asset("workflows-mcp-server", HarnessAssetKind::McpServer),
                ],
                "candidate exposes inputs, stages, assets, approvals, verification, fallbacks, and metrics",
                None,
                true,
            ),
            stage(
                "adversarial-simulation",
                "exercise ambiguous inputs, tool failures, stale evidence, conflicting agents, and unsafe requests",
                vec![
                    asset("workflow-mcp", HarnessAssetKind::McpServer),
                    asset("harness-adversary", HarnessAssetKind::Evaluator),
                ],
                "simulation covers failure paths and verifies fail-closed authority behavior",
                Some("evaluation"),
                false,
            ),
            stage(
                "benchmark",
                "compare candidate behavior against the current harness or a generic-agent baseline",
                vec![asset("harness-benchmark", HarnessAssetKind::Evaluator)],
                "matched tasks report correctness, evidence completion, safety, cost, and latency",
                Some("evaluation"),
                false,
            ),
            stage(
                "promotion-proposal",
                "produce an advisory promotion decision from independent evaluation evidence",
                vec![asset(
                    "superpowers:verification-before-completion",
                    HarnessAssetKind::Skill,
                )],
                "proposal cannot mutate registry or execution authority and cites every gating metric",
                None,
                true,
            ),
            stage(
                "versioned-learning",
                "publish replayable metadata and retain measured outcomes for future routing improvements",
                vec![
                    asset("agentplaybooks", HarnessAssetKind::McpServer),
                    asset("cool-workflow", HarnessAssetKind::Workflow),
                    asset("engram", HarnessAssetKind::Tool),
                ],
                "metadata is versioned, replayable, provenance-bearing, and linked to evaluation evidence",
                None,
                false,
            ),
        ],
        &[
            "correctness versus baseline",
            "evidence completion",
            "unsafe-action rejection",
            "workflow efficiency",
            "replayability",
        ],
        "retain task traces, evaluations, anti-patterns, and version relationships without execution authority",
        "promotion requires no safety/correctness regression and a measured efficiency improvement",
    )
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
