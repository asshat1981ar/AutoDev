//! The logical agent registry.
//!
//! Agents are **logical runtimes**: declarative [`AgentProfile`]s held in an
//! [`AgentRegistry`], not independent processes or microservices. This mirrors
//! the design rule that no unnecessary processes are created — an agent is a
//! set of declared characteristics (role, capabilities, policy, model
//! requirements) that the orchestrator dispatches work to in-process.
//!
//! The registry is the source of truth for *which* agents can do *what*, which
//! lets policy and the orchestrator reason about agents without spawning them.

use crate::action::{Capability, RiskLevel};
use serde::{Deserialize, Serialize};

/// The initial set of agent roles in the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    Architect,
    Researcher,
    Planner,
    Developer,
    Tester,
    SecurityReviewer,
    Release,
}

impl AgentRole {
    /// The wire name of this role.
    pub fn as_str(self) -> &'static str {
        match self {
            AgentRole::Architect => "architect",
            AgentRole::Researcher => "researcher",
            AgentRole::Planner => "planner",
            AgentRole::Developer => "developer",
            AgentRole::Tester => "tester",
            AgentRole::SecurityReviewer => "security_reviewer",
            AgentRole::Release => "release",
        }
    }
}

/// A capability granted to an agent.
///
/// This reuses the platform's [`Capability`] type so the agent registry speaks
/// the same capability language as the policy and execution layers.
pub type AgentCapability = Capability;

/// Model requirements an agent needs to function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRequirement {
    /// The model family/fabric (e.g. "ollama").
    pub family: String,
    /// Preferred model name (e.g. "qwen2.5-coder").
    pub preferred: String,
    /// Minimum context/token budget required.
    pub min_context_tokens: u32,
}

/// The retry policy applied to an agent's actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of attempts (>= 1).
    pub max_attempts: u32,
    /// Base backoff in milliseconds between attempts.
    pub backoff_ms: u64,
}

/// An agent's policy constraints: what it may risk, what tools it may use, and
/// its execution limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPolicy {
    /// The highest risk level this agent may undertake.
    pub risk_ceiling: RiskLevel,
    /// The tools this agent may invoke.
    pub tools: Vec<String>,
    /// Maximum execution timeout in seconds.
    pub timeout_secs: u64,
    /// Retry behavior for this agent's actions.
    pub retry: RetryPolicy,
}

/// The full, declarative definition of an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentProfile {
    /// The agent's role.
    pub role: AgentRole,
    /// Human-readable description of the role.
    pub description: String,
    /// The capabilities this agent is granted.
    pub capabilities: Vec<AgentCapability>,
    /// The model this agent requires.
    pub model: ModelRequirement,
    /// The agent's policy (risk ceiling, tools, timeout, retry).
    pub policy: AgentPolicy,
    /// The expected output shape of this agent's work.
    pub expected_output: String,
}

impl AgentProfile {
    /// Whether this agent may perform an action requiring `cap` and the given
    /// risk level (i.e. it holds the capability and the risk is within its
    /// ceiling).
    pub fn may(&self, cap: &AgentCapability, risk: RiskLevel) -> bool {
        self.capabilities.iter().any(|c| c == cap) && risk_le(&risk, &self.policy.risk_ceiling)
    }
}

/// The health of an agent runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentHealth {
    /// Registered but not yet started.
    Idle,
    /// Currently executing work.
    Running,
    /// Ready and healthy.
    Healthy,
    /// Operating with constraints (e.g. degraded model).
    Degraded,
    /// Unavailable (failed to start or model missing).
    Unavailable,
}

/// The lifecycle state of an agent runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Registered,
    Instantiated,
    Running,
    Healthy,
    Degraded,
    Terminated,
}

/// A logical agent runtime instance bound to a profile.
///
/// This is an in-process handle, not a process. It holds the profile plus
/// runtime state (health, current task, model assignment).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInstance {
    /// Stable identifier for this agent instance.
    pub id: String,
    /// The profile this runtime is bound to.
    pub profile: AgentProfile,
    /// Current lifecycle state.
    pub state: AgentState,
    /// Current health.
    pub health: AgentHealth,
    /// The id of the task currently assigned, if any.
    pub current_task: Option<String>,
    /// The assigned model name, once materialized.
    pub model: Option<String>,
}

/// An append-only registry of agent profiles and runtimes.
#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    profiles: Vec<AgentProfile>,
    runtimes: Vec<AgentInstance>,
}

impl AgentRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        AgentRegistry::default()
    }

    /// Register a profile.
    pub fn register(&mut self, profile: AgentProfile) {
        if !self.profiles.iter().any(|p| p.role == profile.role) {
            self.profiles.push(profile);
        }
    }

    /// All registered profiles.
    pub fn profiles(&self) -> &[AgentProfile] {
        &self.profiles
    }

    /// Look up a profile by role.
    pub fn profile(&self, role: AgentRole) -> Option<&AgentProfile> {
        self.profiles.iter().find(|p| p.role == role)
    }

    /// Instantiate a runtime from a profile by role.
    pub fn instantiate(&mut self, role: AgentRole, id: &str) -> Option<AgentInstance> {
        let profile = self.profile(role)?.clone();
        let runtime = AgentInstance {
            id: id.to_string(),
            profile,
            state: AgentState::Instantiated,
            health: AgentHealth::Idle,
            current_task: None,
            model: None,
        };
        self.runtimes.push(runtime.clone());
        Some(runtime)
    }

    /// Mark a runtime as running on a task.
    pub fn start(&mut self, id: &str, task_id: &str, model: &str) -> Result<(), AgentError> {
        let rt = self
            .runtimes
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(AgentError::UnknownAgent(id.to_string()))?;
        rt.state = AgentState::Running;
        rt.health = AgentHealth::Running;
        rt.current_task = Some(task_id.to_string());
        rt.model = Some(model.to_string());
        Ok(())
    }

    /// Mark a runtime healthy after completing work.
    pub fn mark_healthy(&mut self, id: &str) -> Result<(), AgentError> {
        let rt = self
            .runtimes
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(AgentError::UnknownAgent(id.to_string()))?;
        rt.state = AgentState::Healthy;
        rt.health = AgentHealth::Healthy;
        rt.current_task = None;
        Ok(())
    }

    /// Terminate a runtime.
    pub fn terminate(&mut self, id: &str) -> Result<(), AgentError> {
        let rt = self
            .runtimes
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(AgentError::UnknownAgent(id.to_string()))?;
        rt.state = AgentState::Terminated;
        rt.health = AgentHealth::Unavailable;
        rt.current_task = None;
        Ok(())
    }

    /// All live runtimes.
    pub fn runtimes(&self) -> &[AgentInstance] {
        &self.runtimes
    }

    /// Look up a runtime by id.
    pub fn runtime(&self, id: &str) -> Option<&AgentInstance> {
        self.runtimes.iter().find(|r| r.id == id)
    }
}

/// Errors produced by the agent registry.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AgentError {
    #[error("no agent with id '{0}'")]
    UnknownAgent(String),
}

/// Compare two risk levels for `a <= b`.
fn risk_le(a: &RiskLevel, b: &RiskLevel) -> bool {
    rank(a) <= rank(b)
}

fn rank(r: &RiskLevel) -> u8 {
    match r {
        RiskLevel::Low => 0,
        RiskLevel::Medium => 1,
        RiskLevel::High => 2,
        RiskLevel::Critical => 3,
    }
}
/// Build the initial set of agent profiles for the seven roles.
///
/// Each profile declares its role, capabilities, model requirements, risk
/// ceiling, tools, timeout, retry policy, and expected output.
#[allow(clippy::vec_init_then_push)]
pub fn default_profiles() -> Vec<AgentProfile> {
    let mut profiles = Vec::new();

    profiles.push(AgentProfile {
        role: AgentRole::Architect,
        description: "Designs architecture and reviews structural changes.".to_string(),
        capabilities: vec![
            Capability::ReadFile,
            Capability::WriteFile,
            Capability::PatchFile,
            Capability::Git,
        ],
        model: ModelRequirement {
            family: "ollama".to_string(),
            preferred: "qwen2.5-coder".to_string(),
            min_context_tokens: 8192,
        },
        policy: AgentPolicy {
            risk_ceiling: RiskLevel::High,
            tools: vec!["read_file".to_string(), "git".to_string()],
            timeout_secs: 120,
            retry: RetryPolicy {
                max_attempts: 2,
                backoff_ms: 500,
            },
        },
        expected_output: "architecture_adr".to_string(),
    });

    profiles.push(AgentProfile {
        role: AgentRole::Researcher,
        description: "Gathers information and reads the workspace and history.".to_string(),
        capabilities: vec![Capability::ReadFile, Capability::Git],
        model: ModelRequirement {
            family: "ollama".to_string(),
            preferred: "qwen2.5".to_string(),
            min_context_tokens: 4096,
        },
        policy: AgentPolicy {
            risk_ceiling: RiskLevel::Low,
            tools: vec!["read_file".to_string(), "git".to_string()],
            timeout_secs: 60,
            retry: RetryPolicy {
                max_attempts: 3,
                backoff_ms: 250,
            },
        },
        expected_output: "research_summary".to_string(),
    });

    profiles.push(AgentProfile {
        role: AgentRole::Planner,
        description: "Breaks work into tasks and plans sequences of actions.".to_string(),
        capabilities: vec![Capability::ReadFile, Capability::Git],
        model: ModelRequirement {
            family: "ollama".to_string(),
            preferred: "qwen2.5".to_string(),
            min_context_tokens: 4096,
        },
        policy: AgentPolicy {
            risk_ceiling: RiskLevel::Low,
            tools: vec!["read_file".to_string(), "git".to_string()],
            timeout_secs: 60,
            retry: RetryPolicy {
                max_attempts: 2,
                backoff_ms: 300,
            },
        },
        expected_output: "task_plan".to_string(),
    });

    profiles.push(AgentProfile {
        role: AgentRole::Developer,
        description: "Implements changes using read/write/patch and git.".to_string(),
        capabilities: vec![
            Capability::ReadFile,
            Capability::WriteFile,
            Capability::PatchFile,
            Capability::Git,
            Capability::GitWrite,
        ],
        model: ModelRequirement {
            family: "ollama".to_string(),
            preferred: "qwen2.5-coder".to_string(),
            min_context_tokens: 8192,
        },
        policy: AgentPolicy {
            risk_ceiling: RiskLevel::Medium,
            tools: vec![
                "read_file".to_string(),
                "write_file".to_string(),
                "patch_file".to_string(),
                "git".to_string(),
            ],
            timeout_secs: 180,
            retry: RetryPolicy {
                max_attempts: 3,
                backoff_ms: 500,
            },
        },
        expected_output: "code_change".to_string(),
    });

    profiles.push(AgentProfile {
        role: AgentRole::Tester,
        description: "Runs tests and verifies behavior.".to_string(),
        capabilities: vec![
            Capability::ReadFile,
            Capability::RunTest,
            Capability::Execute,
        ],
        model: ModelRequirement {
            family: "ollama".to_string(),
            preferred: "qwen2.5".to_string(),
            min_context_tokens: 4096,
        },
        policy: AgentPolicy {
            risk_ceiling: RiskLevel::Medium,
            tools: vec![
                "read_file".to_string(),
                "run_test".to_string(),
                "execute".to_string(),
            ],
            timeout_secs: 300,
            retry: RetryPolicy {
                max_attempts: 3,
                backoff_ms: 1000,
            },
        },
        expected_output: "test_report".to_string(),
    });

    profiles.push(AgentProfile {
        role: AgentRole::SecurityReviewer,
        description: "Reviews changes for security and policy compliance.".to_string(),
        capabilities: vec![Capability::ReadFile, Capability::Git, Capability::Execute],
        model: ModelRequirement {
            family: "ollama".to_string(),
            preferred: "qwen2.5".to_string(),
            min_context_tokens: 8192,
        },
        policy: AgentPolicy {
            risk_ceiling: RiskLevel::High,
            tools: vec![
                "read_file".to_string(),
                "git".to_string(),
                "execute".to_string(),
            ],
            timeout_secs: 180,
            retry: RetryPolicy {
                max_attempts: 2,
                backoff_ms: 500,
            },
        },
        expected_output: "security_review".to_string(),
    });

    profiles.push(AgentProfile {
        role: AgentRole::Release,
        description: "Prepares and executes releases (mutating/destructive git).".to_string(),
        capabilities: vec![
            Capability::Git,
            Capability::GitWrite,
            Capability::GitDestructive,
            Capability::ApprovalCritical,
        ],
        model: ModelRequirement {
            family: "ollama".to_string(),
            preferred: "qwen2.5".to_string(),
            min_context_tokens: 4096,
        },
        policy: AgentPolicy {
            risk_ceiling: RiskLevel::High,
            tools: vec!["git".to_string()],
            timeout_secs: 300,
            retry: RetryPolicy {
                max_attempts: 2,
                backoff_ms: 1000,
            },
        },
        expected_output: "release_manifest".to_string(),
    });

    profiles
}
#[cfg(test)]
mod tests {
    use super::*;

    fn default_registry() -> AgentRegistry {
        let mut reg = AgentRegistry::new();
        for p in default_profiles() {
            reg.register(p);
        }
        reg
    }

    #[test]
    fn registers_all_seven_roles() {
        let reg = default_registry();
        assert_eq!(reg.profiles().len(), 7);
        for role in [
            AgentRole::Architect,
            AgentRole::Researcher,
            AgentRole::Planner,
            AgentRole::Developer,
            AgentRole::Tester,
            AgentRole::SecurityReviewer,
            AgentRole::Release,
        ] {
            assert!(reg.profile(role).is_some(), "missing role {role:?}");
        }
    }

    #[test]
    fn each_profile_declares_required_fields() {
        let reg = default_registry();
        for p in reg.profiles() {
            assert!(!p.description.is_empty());
            assert!(!p.capabilities.is_empty());
            assert!(!p.model.family.is_empty());
            assert!(!p.model.preferred.is_empty());
            assert!(p.policy.timeout_secs > 0);
            assert!(p.policy.retry.max_attempts >= 1);
            assert!(!p.expected_output.is_empty());
        }
    }

    #[test]
    fn developer_can_write_but_researcher_cannot() {
        let reg = default_registry();
        let dev = reg.profile(AgentRole::Developer).unwrap();
        let res = reg.profile(AgentRole::Researcher).unwrap();
        assert!(dev.may(&Capability::WriteFile, RiskLevel::Low));
        assert!(!res.may(&Capability::WriteFile, RiskLevel::Low));
    }

    #[test]
    fn risk_ceiling_is_enforced() {
        let reg = default_registry();
        let res = reg.profile(AgentRole::Researcher).unwrap();
        // Researcher ceiling is Low; a High-risk action is refused.
        assert!(!res.may(&Capability::ReadFile, RiskLevel::High));
        assert!(res.may(&Capability::ReadFile, RiskLevel::Low));
    }

    #[test]
    fn architect_has_no_execute_capability() {
        let reg = default_registry();
        let arch = reg.profile(AgentRole::Architect).unwrap();
        assert!(!arch.may(&Capability::Execute, RiskLevel::Low));
    }

    #[test]
    fn release_holds_destructive_git() {
        let reg = default_registry();
        let release = reg.profile(AgentRole::Release).unwrap();
        assert!(release.may(&Capability::GitDestructive, RiskLevel::High));
    }

    #[test]
    fn instantiate_and_lifecycle() {
        let mut reg = default_registry();
        let rt = reg.instantiate(AgentRole::Developer, "dev-1").unwrap();
        assert_eq!(rt.state, AgentState::Instantiated);
        assert_eq!(rt.health, AgentHealth::Idle);

        reg.start("dev-1", "task-9", "qwen2.5-coder").unwrap();
        let running = reg.runtime("dev-1").unwrap();
        assert_eq!(running.state, AgentState::Running);
        assert_eq!(running.current_task.as_deref(), Some("task-9"));

        reg.mark_healthy("dev-1").unwrap();
        let healthy = reg.runtime("dev-1").unwrap();
        assert_eq!(healthy.state, AgentState::Healthy);
        assert!(healthy.current_task.is_none());
    }

    #[test]
    fn unknown_agent_is_reported() {
        let mut reg = default_registry();
        let err = reg.start("nope", "t", "m").unwrap_err();
        assert!(matches!(err, AgentError::UnknownAgent(_)));
    }

    #[test]
    fn duplicate_register_is_idempotent() {
        let mut reg = AgentRegistry::new();
        for p in default_profiles() {
            reg.register(p.clone());
            reg.register(p);
        }
        assert_eq!(reg.profiles().len(), 7);
    }
}
