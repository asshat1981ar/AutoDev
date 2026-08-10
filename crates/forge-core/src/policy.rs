//! Policy evaluation for agent actions.
//!
//! The policy layer sits between the (untrusted) agent intent and the
//! (trusted) execution adapters. It performs structural validation, capability
//! checks, and risk-based authorization. No privileged operation occurs until
//! policy authorizes it.

use crate::action::{AgentAction, Capability, RiskLevel};
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
