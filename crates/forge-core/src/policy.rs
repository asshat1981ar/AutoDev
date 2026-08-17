//! Policy evaluation for agent actions.
//!
//! The policy layer sits between the (untrusted) agent intent and the
//! (trusted) execution adapters. It performs structural validation, capability
//! checks, and risk-based authorization. No privileged operation occurs until
//! policy authorizes it.

use crate::action::{ActionType, AgentAction, Capability, RiskLevel};
use crate::error::ExecutionError;

/// The outcome of policy evaluation for an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    /// The action is authorized and may proceed.
    Allow,
    /// The action requires human approval before execution.
    RequireApproval,
    /// The action is denied; it must not be executed.
    Deny,
}

/// Kernel-owned authorization material. Unlike `AgentAction.capabilities`, this
/// value is never supplied by the model; it is created by the trusted
/// orchestration boundary after an approval decision has been recorded.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AuthorizationGrant {
    pub approval_ref: Option<String>,
}

impl AuthorizationGrant {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn approved(reference: impl Into<String>) -> Self {
        Self {
            approval_ref: Some(reference.into()),
        }
    }

    pub fn is_approved(&self) -> bool {
        self.approval_ref
            .as_deref()
            .is_some_and(|reference| !reference.trim().is_empty())
    }
}

/// Trusted minimum risk for an executable action.
///
/// Model-declared risk is advisory. This floor is derived from action semantics
/// inside ForgeCore so an untrusted producer cannot downgrade an operation to
/// bypass approval.
pub fn minimum_risk_for_action(action: &AgentAction) -> RiskLevel {
    match action.action_type {
        ActionType::ReadFile | ActionType::RequestApproval => RiskLevel::Low,
        ActionType::WriteFile | ActionType::PatchFile | ActionType::RunTest => RiskLevel::Medium,
        ActionType::Execute | ActionType::Mcp => RiskLevel::High,
        ActionType::Git => match action
            .payload
            .get("operation")
            .and_then(serde_json::Value::as_str)
        {
            Some("checkpoint" | "prepare_commit") => RiskLevel::Medium,
            Some("rollback") => RiskLevel::High,
            _ => RiskLevel::Low,
        },
    }
}

/// Effective authorization risk after applying the trusted semantic floor.
pub fn effective_risk_for_action(action: &AgentAction) -> RiskLevel {
    let minimum = minimum_risk_for_action(action);
    if risk_rank(action.risk) >= risk_rank(minimum) {
        action.risk
    } else {
        minimum
    }
}

fn risk_rank(risk: RiskLevel) -> u8 {
    match risk {
        RiskLevel::Low => 0,
        RiskLevel::Medium => 1,
        RiskLevel::High => 2,
        RiskLevel::Critical => 3,
    }
}

/// Validate the structural invariants that must hold before policy evaluation.
pub fn validate_action(action: &AgentAction) -> Result<(), ExecutionError> {
    if action.id.trim().is_empty() {
        return Err(ExecutionError::MissingActionId);
    }
    if action.task_id.trim().is_empty() {
        return Err(ExecutionError::MissingTaskId);
    }
    if action.agent_id.trim().is_empty() {
        return Err(ExecutionError::MissingAgentId);
    }
    if action.reason.trim().is_empty() {
        return Err(ExecutionError::MissingReason);
    }
    if action.risk == RiskLevel::Critical
        && !action
            .capabilities
            .iter()
            .any(|c| c == &Capability::ApprovalCritical)
    {
        return Err(ExecutionError::CriticalApprovalRequired);
    }
    Ok(())
}

/// Check that the action type maps to a capability the agent has been granted.
///
/// Returns `true` when the action's required capability is present in the
/// granted set, or when the action type has no capability requirement.
pub fn has_required_capability(action: &AgentAction) -> bool {
    match Capability::for_action(action.action_type) {
        Some(required) => action.capabilities.iter().any(|c| c == &required),
        None => true,
    }
}

/// The capability required by an action, if any.
pub fn required_capability(action: &AgentAction) -> Option<Capability> {
    Capability::for_action(action.action_type)
}

/// Evaluate the full policy for an action.
///
/// Performs structural validation, then applies risk-based authorization. The
/// capability check is separate (`has_required_capability`) so that a denied
/// action can be distinguished from one that merely requires approval.
pub fn evaluate_policy(action: &AgentAction) -> Result<PolicyDecision, ExecutionError> {
    validate_action(action)?;

    Ok(match action.risk {
        RiskLevel::Low => PolicyDecision::Allow,
        RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical => {
            PolicyDecision::RequireApproval
        }
    })
}

/// Resolve a policy decision against a trusted authorization grant.
///
/// An agent cannot satisfy approval by changing its payload or capabilities;
/// only a kernel-created `AuthorizationGrant` can discharge a
/// `RequireApproval` decision.
pub fn enforce_policy(
    action: &AgentAction,
    grant: &AuthorizationGrant,
) -> Result<PolicyDecision, ExecutionError> {
    match evaluate_policy(action)? {
        PolicyDecision::Allow => Ok(PolicyDecision::Allow),
        PolicyDecision::RequireApproval if grant.is_approved() => {
            Ok(PolicyDecision::RequireApproval)
        }
        PolicyDecision::RequireApproval => Err(ExecutionError::RequiresApproval),
        PolicyDecision::Deny => Err(ExecutionError::CapabilityDenied),
    }
}
