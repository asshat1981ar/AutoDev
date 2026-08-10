//! The agent runtime: connects an agent to tasks, models, tools, and typed
//! actions.
//!
//! An [`AgentRuntime`] is bound to an [`AgentProfile`] and drives a single
//! step through the lifecycle:
//!
//! ```text
//! CREATED → READY → PLANNING → ACTING → WAITING
//!   → VERIFYING → COMPLETED
//!
//! Failure states: FAILED, BLOCKED, CANCELLED
//! ```
//!
//! The runtime loop, per step:
//!
//! 1. receive task context
//! 2. assemble agent context (task + profile + capabilities)
//! 3. select model (provider routing or the agent's preferred model)
//! 4. invoke model (via the provider-neutral [`ModelProvider`])
//! 5. validate structured output (parse into an [`AgentAction`])
//! 6. produce AgentAction
//! 7. submit action to policy (validate + capability + risk)
//! 8. consume ExecutionResult (execute, record evidence, feed back)
//!
//! The runtime is in-process (no processes are spawned) and is testable with a
//! [`MockProvider`] and a mock executor, so orchestration can be exercised
//! without a real model.

use crate::action::{AgentAction, Capability, RiskLevel};
use crate::agent::{AgentProfile, AgentRole};
use crate::error::ExecutionError;
use crate::evidence::{
    record_from, EvidenceStore, ExecutionResult, ExecutionStatus, PolicyOutcome,
};
use crate::model::{ModelProvider, ModelRequest, ModelResponse};
use crate::policy::{evaluate_policy, has_required_capability, PolicyDecision};
use serde::{Deserialize, Serialize};

/// The lifecycle state of an agent runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentRuntimeState {
    Created,
    Ready,
    Planning,
    Acting,
    Waiting,
    Verifying,
    Completed,
    Failed,
    Blocked,
    Cancelled,
}

/// A task assigned to an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    /// The task description / context given to the agent.
    pub context: String,
}

/// The validated structured output produced by a model, ready to become an
/// [`AgentAction`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredOutput {
    /// The action type (serialized wire name, e.g. "read_file").
    pub action: String,
    /// The reason for the action.
    pub reason: String,
    /// The risk level (serialized, e.g. "low").
    pub risk: String,
    /// The action payload.
    pub payload: serde_json::Value,
}

/// The result of a runtime step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepOutcome {
    /// The state the runtime is in after this step.
    pub state: AgentRuntimeState,
    /// The action produced, if any.
    pub action: Option<AgentAction>,
    /// The execution result consumed, if any.
    pub result: Option<ExecutionResult>,
    /// A human/task-facing message.
    pub message: String,
}

/// Errors produced by the agent runtime.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RuntimeError {
    #[error("no model available for agent '{0}'")]
    NoModel(String),
    #[error("model output could not be parsed into an action: {0}")]
    InvalidOutput(String),
    #[error("policy refused the action: {0}")]
    PolicyDenied(String),
    #[error("no model provider configured")]
    NoProvider,
    #[error("no action was produced to execute")]
    NoAction,
    #[error("action execution failed: {0}")]
    ExecutionFailed(String),
}

/// A function that executes an [`AgentAction`] and returns a result.
///
/// This indirection lets the runtime be tested with a mock executor (no real
/// filesystem/git) and lets orchestration inject the real `execute()` bound to
/// a workspace. It is a boxed closure so it can capture a [`crate::Workspace`].
pub type Executor = Box<dyn Fn(&AgentAction) -> Result<ExecutionResult, ExecutionError>>;

/// An agent runtime instance.
///
/// Holds the agent profile, the model provider, the evidence store, and the
/// in-flight task. All heavy work (model calls, execution) is delegated to the
/// injected provider/executor so the runtime is testable without a real model.
pub struct AgentRuntime {
    /// The agent's stable id.
    pub id: String,
    /// The role bound to this runtime.
    pub role: AgentRole,
    /// The profile this runtime is bound to.
    pub profile: AgentProfile,
    /// The current lifecycle state.
    pub state: AgentRuntimeState,
    /// The model provider used to invoke the model.
    provider: Box<dyn ModelProvider>,
    /// The evidence store where results are recorded.
    pub evidence: EvidenceStore,
    /// The executor used to run actions.
    executor: Executor,
    /// The currently assigned task, if any.
    pub current_task: Option<Task>,
    /// The last action produced, if any.
    pub last_action: Option<AgentAction>,
    /// The last result consumed, if any.
    pub last_result: Option<ExecutionResult>,
    /// The model selected for the current step.
    pub selected_model: Option<String>,
}
impl AgentRuntime {
    /// Create a runtime bound to `profile` using `provider` for model calls and
    /// `executor` for action execution.
    pub fn new(
        id: impl Into<String>,
        profile: AgentProfile,
        provider: Box<dyn ModelProvider>,
        executor: Executor,
    ) -> Self {
        let role = profile.role;
        AgentRuntime {
            id: id.into(),
            role,
            profile,
            state: AgentRuntimeState::Ready,
            provider,
            evidence: EvidenceStore::new(),
            executor,
            current_task: None,
            last_action: None,
            last_result: None,
            selected_model: None,
        }
    }

    /// Receive a task and move to `PLANNING`.
    pub fn assign_task(&mut self, task: Task) {
        self.current_task = Some(task);
        self.state = AgentRuntimeState::Planning;
    }

    /// The current lifecycle state.
    pub fn state(&self) -> AgentRuntimeState {
        self.state
    }

    /// Transition to a new lifecycle state, applying the allowed transitions.
    ///
    /// Returns `false` (without changing state) if the transition is not
    /// allowed, so the runtime maintains a well-formed state machine.
    pub fn transition(&mut self, next: AgentRuntimeState) -> bool {
        // Identity transitions are always allowed (no-op).
        if self.state == next {
            return true;
        }
        let allowed = matches!(
            (self.state, next),
            (AgentRuntimeState::Created, AgentRuntimeState::Ready)
                | (AgentRuntimeState::Ready, AgentRuntimeState::Planning)
                | (AgentRuntimeState::Planning, AgentRuntimeState::Acting)
                | (AgentRuntimeState::Acting, AgentRuntimeState::Waiting)
                | (AgentRuntimeState::Waiting, AgentRuntimeState::Verifying)
                | (AgentRuntimeState::Verifying, AgentRuntimeState::Completed)
                | (AgentRuntimeState::Ready, AgentRuntimeState::Cancelled)
                | (AgentRuntimeState::Planning, AgentRuntimeState::Cancelled)
                | (AgentRuntimeState::Acting, AgentRuntimeState::Cancelled)
                | (AgentRuntimeState::Waiting, AgentRuntimeState::Cancelled)
                | (AgentRuntimeState::Verifying, AgentRuntimeState::Cancelled)
                | (AgentRuntimeState::Acting, AgentRuntimeState::Blocked)
                | (AgentRuntimeState::Waiting, AgentRuntimeState::Blocked)
                | (AgentRuntimeState::Verifying, AgentRuntimeState::Failed)
                | (AgentRuntimeState::Acting, AgentRuntimeState::Failed)
                | (AgentRuntimeState::Planning, AgentRuntimeState::Failed)
                | (AgentRuntimeState::Failed, AgentRuntimeState::Ready)
                | (AgentRuntimeState::Blocked, AgentRuntimeState::Ready)
                | (AgentRuntimeState::Blocked, AgentRuntimeState::Acting)
        );
        if allowed {
            self.state = next;
        }
        allowed
    }

    /// Assemble the agent context (task + role + capabilities) into a prompt.
    pub fn assemble_context(&self) -> String {
        let task = self
            .current_task
            .as_ref()
            .map(|t| format!("Task: {} — {}", t.title, t.context))
            .unwrap_or_else(|| "No task".to_string());
        let caps: Vec<&str> = self
            .profile
            .capabilities
            .iter()
            .map(|c| match c {
                Capability::Unknown(s) => s.as_str(),
                _ => c.as_str(),
            })
            .collect();
        format!(
            "You are the {role} agent.\n{caps:?}\n{task}\nReturn one structured action.",
            role = self.profile.role.as_str()
        )
    }

    /// Select a model: prefer the agent's preferred model; fall back to the
    /// first provider model that supports chat.
    pub fn select_model(&mut self) -> Result<String, RuntimeError> {
        let preferred = self.profile.model.preferred.clone();
        let models = self
            .provider
            .list_models()
            .map_err(|e| RuntimeError::NoModel(e.to_string()))?;
        if models.iter().any(|m| m.id == preferred) {
            self.selected_model = Some(preferred.clone());
            return Ok(preferred);
        }
        let first_chat = models
            .iter()
            .find(|m| m.capabilities.chat)
            .map(|m| m.id.clone());
        match first_chat {
            Some(id) => {
                self.selected_model = Some(id.clone());
                Ok(id)
            }
            None => Err(RuntimeError::NoModel(self.id.clone())),
        }
    }

    /// Invoke the model and return the raw response.
    pub fn invoke_model(&self, model: &str) -> Result<ModelResponse, RuntimeError> {
        let context = self.assemble_context();
        let req = ModelRequest {
            model: model.to_string(),
            messages: Some(vec![crate::model::Message {
                role: "user".to_string(),
                content: context,
            }]),
            prompt: None,
            options: None,
        };
        self.provider
            .chat(&req)
            .map_err(|e| RuntimeError::ExecutionFailed(e.to_string()))
    }
}
impl AgentRuntime {
    /// Validate the model's structured output into an [`AgentAction`].
    pub fn validate_output(&self, response: &ModelResponse) -> Result<AgentAction, RuntimeError> {
        let parsed: StructuredOutput = serde_json::from_str(&response.content)
            .map_err(|e| RuntimeError::InvalidOutput(e.to_string()))?;
        let action_type = parse_action_type(&parsed.action)
            .ok_or_else(|| RuntimeError::InvalidOutput("unknown action type".into()))?;
        let risk = parse_risk(&parsed.risk)
            .ok_or_else(|| RuntimeError::InvalidOutput("unknown risk".into()))?;
        let task = self
            .current_task
            .as_ref()
            .ok_or_else(|| RuntimeError::InvalidOutput("no task".into()))?;
        Ok(AgentAction {
            id: format!("{}-{}", self.id, task.id),
            task_id: task.id.clone(),
            agent_id: self.id.clone(),
            action_type,
            reason: parsed.reason,
            risk,
            capabilities: self.profile.capabilities.clone(),
            payload: parsed.payload,
            expected: serde_json::json!({}),
        })
    }

    /// Submit an action to policy. Returns the decision.
    pub fn submit_to_policy(&self, action: &AgentAction) -> Result<PolicyDecision, RuntimeError> {
        evaluate_policy(action).map_err(|e| RuntimeError::PolicyDenied(e.to_string()))?;
        if !has_required_capability(action) {
            return Err(RuntimeError::PolicyDenied("missing capability".into()));
        }
        Ok(match action.risk {
            RiskLevel::Low => PolicyDecision::Allow,
            RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical => {
                PolicyDecision::RequireApproval
            }
        })
    }

    /// Consume an [`ExecutionResult`]: record it as evidence and store it back.
    pub fn consume_result(&mut self, action: &AgentAction, result: &ExecutionResult) {
        let record = record_from(
            &format!("{}-rec", result.action_id),
            action,
            outcome_from_status(result.status),
            result,
            vec![],
        );
        self.evidence.insert(record);
        self.last_result = Some(result.clone());
    }

    /// Run a single step: receive context, plan, act, verify.
    ///
    /// This is the core loop, exercised by the tests. It invokes the model,
    /// validates the output into an action, submits it to policy, executes it
    /// (via the injected executor), and records evidence.
    pub fn run_step(&mut self) -> Result<StepOutcome, RuntimeError> {
        if self.current_task.is_none() {
            return Err(RuntimeError::NoAction);
        }
        self.transition(AgentRuntimeState::Planning);

        // Select + invoke model.
        let model = self.select_model()?;
        self.transition(AgentRuntimeState::Acting);
        let response = self.invoke_model(&model)?;

        // Validate output -> AgentAction.
        let action = self.validate_output(&response)?;
        self.last_action = Some(action.clone());

        // Submit to policy.
        let decision = self.submit_to_policy(&action)?;
        if decision == PolicyDecision::RequireApproval {
            self.transition(AgentRuntimeState::Blocked);
            return Ok(StepOutcome {
                state: self.state,
                action: Some(action),
                result: None,
                message: "action requires approval".to_string(),
            });
        }

        // Execute, then move through WAITING to VERIFYING.
        self.transition(AgentRuntimeState::Waiting);
        let result =
            (self.executor)(&action).map_err(|e| RuntimeError::ExecutionFailed(e.to_string()))?;
        self.consume_result(&action, &result);
        self.transition(AgentRuntimeState::Verifying);

        let ok = matches!(
            result.status,
            ExecutionStatus::Succeeded | ExecutionStatus::Accepted
        );
        self.transition(if ok {
            AgentRuntimeState::Completed
        } else {
            AgentRuntimeState::Failed
        });

        Ok(StepOutcome {
            state: self.state,
            action: Some(action),
            result: Some(result),
            message: "step complete".to_string(),
        })
    }
}

/// Parse an [`ActionType`] from its wire name.
fn parse_action_type(s: &str) -> Option<crate::action::ActionType> {
    use crate::action::ActionType;
    Some(match s {
        "read_file" => ActionType::ReadFile,
        "write_file" => ActionType::WriteFile,
        "patch_file" => ActionType::PatchFile,
        "execute" => ActionType::Execute,
        "git" => ActionType::Git,
        "mcp" => ActionType::Mcp,
        "run_test" => ActionType::RunTest,
        "request_approval" => ActionType::RequestApproval,
        _ => return None,
    })
}

/// Parse a [`RiskLevel`] from its wire name.
fn parse_risk(s: &str) -> Option<RiskLevel> {
    Some(match s {
        "low" => RiskLevel::Low,
        "medium" => RiskLevel::Medium,
        "high" => RiskLevel::High,
        "critical" => RiskLevel::Critical,
        _ => return None,
    })
}

/// Map an [`ExecutionStatus`] to a [`PolicyOutcome`].
fn outcome_from_status(status: ExecutionStatus) -> PolicyOutcome {
    match status {
        ExecutionStatus::Accepted | ExecutionStatus::Succeeded => PolicyOutcome::Allow,
        ExecutionStatus::Running => PolicyOutcome::Allow,
        ExecutionStatus::Denied | ExecutionStatus::Failed => PolicyOutcome::Deny,
        ExecutionStatus::Cancelled => PolicyOutcome::RequireApproval,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::default_profiles;
    use crate::model::{MockProvider, ModelHealth};

    /// A mock executor that always succeeds with a canned result.
    fn mock_executor(action: &AgentAction) -> Result<ExecutionResult, ExecutionError> {
        let now = chrono::Utc::now();
        Ok(ExecutionResult {
            action_id: action.id.clone(),
            status: ExecutionStatus::Succeeded,
            started_at: now,
            completed_at: now,
            exit_code: None,
            stdout: "ok".to_string(),
            stderr: String::new(),
            artifacts: vec![],
            verification: None,
            error: None,
        })
    }

    fn runtime_with(provider: Box<dyn ModelProvider>) -> AgentRuntime {
        let profile = default_profiles()
            .into_iter()
            .find(|p| p.role == AgentRole::Developer)
            .unwrap();
        AgentRuntime::new("dev-rt", profile, provider, Box::new(mock_executor))
    }

    fn task() -> Task {
        Task {
            id: "t1".to_string(),
            title: "fix".to_string(),
            context: "fix the bug".to_string(),
        }
    }

    #[test]
    fn lifecycle_starts_ready_and_assigns_task() {
        let mut rt = runtime_with(Box::new(MockProvider::default()));
        assert_eq!(rt.state, AgentRuntimeState::Ready);
        rt.assign_task(task());
        assert_eq!(rt.state, AgentRuntimeState::Planning);
    }

    #[test]
    fn assemble_context_includes_role_and_task() {
        let mut rt = runtime_with(Box::new(MockProvider::default()));
        rt.assign_task(task());
        let ctx = rt.assemble_context();
        assert!(ctx.contains("developer"));
        assert!(ctx.contains("fix the bug"));
    }

    #[test]
    fn selects_model_from_provider() {
        let mut rt = runtime_with(Box::new(MockProvider::new("{}")));
        rt.assign_task(task());
        let model = rt.select_model().unwrap();
        assert_eq!(model, "mock-model");
        assert_eq!(rt.selected_model.as_deref(), Some("mock-model"));
    }
    #[test]
    fn validates_structured_output_into_action() {
        let mut rt = runtime_with(Box::new(MockProvider::default()));
        rt.assign_task(task());
        let response = ModelResponse {
            model: "mock-model".to_string(),
            content: serde_json::json!({
                "action": "read_file",
                "reason": "inspect",
                "risk": "low",
                "payload": { "path": "a.txt" }
            })
            .to_string(),
            usage: crate::model::Usage::default(),
            load_ns: 0,
            eval_ns: 0,
            provider: "mock".to_string(),
        };
        let action = rt.validate_output(&response).unwrap();
        assert_eq!(action.action_type, crate::action::ActionType::ReadFile);
        assert_eq!(action.task_id, "t1");
        assert_eq!(action.agent_id, "dev-rt");
        assert!(action
            .capabilities
            .iter()
            .any(|c| c == &Capability::ReadFile));
    }

    #[test]
    fn rejects_malformed_output() {
        let mut rt = runtime_with(Box::new(MockProvider::default()));
        rt.assign_task(task());
        let response = ModelResponse {
            model: "m".to_string(),
            content: "not json".to_string(),
            usage: crate::model::Usage::default(),
            load_ns: 0,
            eval_ns: 0,
            provider: "mock".to_string(),
        };
        let err = rt.validate_output(&response).unwrap_err();
        assert!(matches!(err, RuntimeError::InvalidOutput(_)));
    }

    #[test]
    fn run_step_executes_and_records_evidence() {
        let provider = MockProvider::new(
            serde_json::json!({
                "action": "read_file",
                "reason": "inspect",
                "risk": "low",
                "payload": { "path": "a.txt" }
            })
            .to_string(),
        );
        let mut rt = runtime_with(Box::new(provider));
        rt.assign_task(task());
        let outcome = rt.run_step().unwrap();
        assert_eq!(outcome.state, AgentRuntimeState::Completed);
        assert!(outcome.action.is_some());
        assert!(outcome.result.is_some());
        assert_eq!(rt.evidence.len(), 1);
        assert!(rt.evidence.records()[0].verify());
    }

    #[test]
    fn high_risk_action_blocks_for_approval() {
        let provider = MockProvider::new(
            serde_json::json!({
                "action": "write_file",
                "reason": "write",
                "risk": "high",
                "payload": { "path": "a.txt", "content": "x" }
            })
            .to_string(),
        );
        let mut rt = runtime_with(Box::new(provider));
        rt.assign_task(task());
        let outcome = rt.run_step().unwrap();
        assert_eq!(outcome.state, AgentRuntimeState::Blocked);
        assert!(outcome.result.is_none());
        assert_eq!(rt.evidence.len(), 0);
    }

    #[test]
    fn model_health_is_reported() {
        let provider = MockProvider::default();
        assert_eq!(provider.health(), ModelHealth::Healthy);
    }

    #[test]
    fn state_transitions_are_validated() {
        let mut rt = runtime_with(Box::new(MockProvider::default()));
        // Ready -> Planning is allowed.
        assert!(rt.transition(AgentRuntimeState::Planning));
        assert_eq!(rt.state(), AgentRuntimeState::Planning);
        // Planning -> Completed is NOT a valid transition.
        assert!(!rt.transition(AgentRuntimeState::Completed));
        assert_eq!(rt.state(), AgentRuntimeState::Planning);
    }

    #[test]
    fn run_step_honors_full_lifecycle() {
        let provider = MockProvider::new(
            serde_json::json!({
                "action": "read_file",
                "reason": "inspect",
                "risk": "low",
                "payload": { "path": "a.txt" }
            })
            .to_string(),
        );
        let mut rt = runtime_with(Box::new(provider));
        rt.assign_task(task());
        let outcome = rt.run_step().unwrap();
        assert_eq!(outcome.state, AgentRuntimeState::Completed);
        assert_eq!(rt.state(), AgentRuntimeState::Completed);
    }

    #[test]
    fn agents_never_directly_access_privileged_execution() {
        // Count how many times the executor is invoked.
        let executions = std::rc::Rc::new(std::cell::Cell::new(0usize));
        let counter = std::rc::Rc::clone(&executions);
        let executor: Executor = Box::new(move |_action| {
            counter.set(counter.get() + 1);
            let now = chrono::Utc::now();
            Ok(ExecutionResult {
                action_id: "x".to_string(),
                status: ExecutionStatus::Succeeded,
                started_at: now,
                completed_at: now,
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                artifacts: vec![],
                verification: None,
                error: None,
            })
        });

        // A high-risk action is blocked before execution -> executor never runs.
        let provider = MockProvider::new(
            serde_json::json!({
                "action": "write_file",
                "reason": "write",
                "risk": "high",
                "payload": { "path": "a.txt", "content": "x" }
            })
            .to_string(),
        );
        let profile = default_profiles()
            .into_iter()
            .find(|p| p.role == AgentRole::Developer)
            .unwrap();
        let mut rt = AgentRuntime::new("dev-rt", profile, Box::new(provider), executor);
        rt.assign_task(task());
        let outcome = rt.run_step().unwrap();
        assert_eq!(outcome.state, AgentRuntimeState::Blocked);
        // The agent (model output) never reached the privileged executor.
        assert_eq!(executions.get(), 0);
    }
}
