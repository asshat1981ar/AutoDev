//! Execution envelope binding task intent, context, policy, evidence, and lifecycle.
//!
//! This module is intentionally small: it composes existing ForgeCore protocol
//! types instead of introducing another execution subsystem. The envelope is the
//! durable hand-off object between orchestration phases and future MCP frontends.

use serde::{Deserialize, Serialize};

use crate::{AgentAction, Capability, RiskLevel};

/// References to bounded repository context artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextRefs {
    pub repo_map_ref: Option<String>,
    pub context_pack_ref: Option<String>,
    pub context_delta_ref: Option<String>,
    pub change_impact_ref: Option<String>,
    pub feature_contract_ref: Option<String>,
}

/// Authorization state bound to an execution request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyBinding {
    pub risk: RiskLevel,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    pub requires_approval: bool,
    pub approval_ref: Option<String>,
}

impl PolicyBinding {
    /// Whether this binding is sufficiently authorized to enter execution.
    pub fn is_authorized(&self) -> bool {
        !self.requires_approval || self.approval_ref.as_deref().is_some_and(|r| !r.is_empty())
    }
}

/// Evidence requirements and produced evidence handles.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceBinding {
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub produced: Vec<String>,
}

/// Durable execution-envelope lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeState {
    Planned,
    Authorized,
    Executing,
    Verifying,
    Verified,
    Rejected,
    Replanning,
}

impl EnvelopeState {
    /// Whether a direct lifecycle transition is permitted.
    pub fn can_transition_to(self, next: EnvelopeState) -> bool {
        matches!(
            (self, next),
            (EnvelopeState::Planned, EnvelopeState::Authorized)
                | (EnvelopeState::Planned, EnvelopeState::Rejected)
                | (EnvelopeState::Authorized, EnvelopeState::Executing)
                | (EnvelopeState::Executing, EnvelopeState::Verifying)
                | (EnvelopeState::Executing, EnvelopeState::Rejected)
                | (EnvelopeState::Verifying, EnvelopeState::Verified)
                | (EnvelopeState::Verifying, EnvelopeState::Rejected)
                | (EnvelopeState::Rejected, EnvelopeState::Replanning)
                | (EnvelopeState::Replanning, EnvelopeState::Planned)
        )
    }
}

/// Retry/lifecycle metadata for a durable execution attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lifecycle {
    pub attempt: u32,
    pub max_attempts: u32,
    pub state: EnvelopeState,
}

impl Lifecycle {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            attempt: 1,
            max_attempts: max_attempts.max(1),
            state: EnvelopeState::Planned,
        }
    }

    pub fn exhausted(&self) -> bool {
        self.attempt >= self.max_attempts
    }
}

/// The typed hand-off object used across PLAN -> AUTHORIZE -> ACT -> VERIFY.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionEnvelope {
    pub operation_id: String,
    pub run_id: String,
    pub task_id: String,
    pub action: AgentAction,
    #[serde(default)]
    pub context: ContextRefs,
    pub policy: PolicyBinding,
    #[serde(default)]
    pub evidence: EvidenceBinding,
    pub lifecycle: Lifecycle,
}

/// Envelope invariant violations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EnvelopeError {
    #[error("operation_id, run_id, and task_id must be non-empty")]
    MissingIdentity,
    #[error("action task_id must match envelope task_id")]
    TaskMismatch,
    #[error("authorization is incomplete for an approval-gated action")]
    ApprovalRequired,
    #[error("invalid lifecycle transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: EnvelopeState,
        to: EnvelopeState,
    },
    #[error("maximum execution attempts exhausted")]
    AttemptsExhausted,
}

impl ExecutionEnvelope {
    /// Validate cross-object invariants that serialization alone cannot express.
    pub fn validate(&self) -> Result<(), EnvelopeError> {
        if self.operation_id.is_empty() || self.run_id.is_empty() || self.task_id.is_empty() {
            return Err(EnvelopeError::MissingIdentity);
        }
        if self.action.task_id != self.task_id {
            return Err(EnvelopeError::TaskMismatch);
        }
        if matches!(
            self.lifecycle.state,
            EnvelopeState::Authorized
                | EnvelopeState::Executing
                | EnvelopeState::Verifying
                | EnvelopeState::Verified
        ) && !self.policy.is_authorized()
        {
            return Err(EnvelopeError::ApprovalRequired);
        }
        Ok(())
    }

    /// Move the envelope through one explicit lifecycle transition.
    pub fn transition(&mut self, next: EnvelopeState) -> Result<(), EnvelopeError> {
        let current = self.lifecycle.state;
        if !current.can_transition_to(next) {
            return Err(EnvelopeError::InvalidTransition {
                from: current,
                to: next,
            });
        }
        if next == EnvelopeState::Authorized && !self.policy.is_authorized() {
            return Err(EnvelopeError::ApprovalRequired);
        }
        if current == EnvelopeState::Replanning && next == EnvelopeState::Planned {
            if self.lifecycle.exhausted() {
                return Err(EnvelopeError::AttemptsExhausted);
            }
            self.lifecycle.attempt += 1;
        }
        self.lifecycle.state = next;
        self.validate()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{ActionType, AgentAction};

    fn action(task_id: &str) -> AgentAction {
        AgentAction {
            id: "a-1".into(),
            task_id: task_id.into(),
            agent_id: "developer".into(),
            action_type: ActionType::WriteFile,
            reason: "test".into(),
            risk: RiskLevel::High,
            capabilities: vec![Capability::WriteFile],
            payload: json!({"path":"src/lib.rs","content":"x"}),
            expected: json!({}),
        }
    }

    fn envelope() -> ExecutionEnvelope {
        ExecutionEnvelope {
            operation_id: "op-1".into(),
            run_id: "run-1".into(),
            task_id: "task-1".into(),
            action: action("task-1"),
            context: ContextRefs::default(),
            policy: PolicyBinding {
                risk: RiskLevel::High,
                capabilities: vec![Capability::WriteFile],
                requires_approval: true,
                approval_ref: None,
            },
            evidence: EvidenceBinding::default(),
            lifecycle: Lifecycle::new(2),
        }
    }

    #[test]
    fn approval_gate_blocks_authorization() {
        let mut env = envelope();
        assert_eq!(
            env.transition(EnvelopeState::Authorized),
            Err(EnvelopeError::ApprovalRequired)
        );
        assert_eq!(env.lifecycle.state, EnvelopeState::Planned);
    }

    #[test]
    fn authorized_flow_reaches_verified() {
        let mut env = envelope();
        env.policy.approval_ref = Some("approval-1".into());
        env.transition(EnvelopeState::Authorized).unwrap();
        env.transition(EnvelopeState::Executing).unwrap();
        env.transition(EnvelopeState::Verifying).unwrap();
        env.transition(EnvelopeState::Verified).unwrap();
        assert_eq!(env.lifecycle.state, EnvelopeState::Verified);
    }

    #[test]
    fn replanning_is_bounded_by_attempt_limit() {
        let mut env = envelope();
        env.policy.approval_ref = Some("approval-1".into());
        env.transition(EnvelopeState::Authorized).unwrap();
        env.transition(EnvelopeState::Executing).unwrap();
        env.transition(EnvelopeState::Rejected).unwrap();
        env.transition(EnvelopeState::Replanning).unwrap();
        env.transition(EnvelopeState::Planned).unwrap();
        assert_eq!(env.lifecycle.attempt, 2);

        env.transition(EnvelopeState::Authorized).unwrap();
        env.transition(EnvelopeState::Executing).unwrap();
        env.transition(EnvelopeState::Rejected).unwrap();
        env.transition(EnvelopeState::Replanning).unwrap();
        assert_eq!(
            env.transition(EnvelopeState::Planned),
            Err(EnvelopeError::AttemptsExhausted)
        );
    }

    #[test]
    fn rejects_task_identity_mismatch() {
        let mut env = envelope();
        env.action.task_id = "other".into();
        assert_eq!(env.validate(), Err(EnvelopeError::TaskMismatch));
    }
}
