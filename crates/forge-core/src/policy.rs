//! Policy evaluation for agent actions.
//!
//! The policy layer sits between untrusted agent intent and trusted execution.
//! `AgentAction` fields are never execution authority. Effective capabilities
//! and approval evidence enter execution through [`ExecutionAuthority`].

use crate::action::{ActionType, AgentAction, Capability, RiskLevel};
use crate::authority::{ExecutionAuthority, GrantedCapability};
use crate::error::ExecutionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    RequireApproval,
    Deny,
}

/// Validate structural invariants of untrusted intent only.
///
/// This function deliberately does not interpret `AgentAction.capabilities` as
/// authorization and does not allow `approval:critical` to satisfy approval.
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
    Ok(())
}

/// Return the kernel capability required to execute an action.
pub fn required_grant(action: &AgentAction) -> Option<GrantedCapability> {
    Some(match action.action_type {
        ActionType::ReadFile => GrantedCapability::ReadFile,
        ActionType::WriteFile => GrantedCapability::WriteFile,
        ActionType::PatchFile => GrantedCapability::PatchFile,
        ActionType::Execute => GrantedCapability::Execute,
        ActionType::Git => GrantedCapability::Git,
        ActionType::Mcp => GrantedCapability::Mcp,
        ActionType::RunTest => GrantedCapability::RunTest,
        ActionType::RequestApproval => GrantedCapability::RequestApproval,
    })
}

/// Check trusted execution authority, never action-carried capability claims.
pub fn has_required_capability(action: &AgentAction, authority: &ExecutionAuthority) -> bool {
    required_grant(action)
        .map(|required| authority.allows(required))
        .unwrap_or(true)
}

/// Protocol-level capability required by an action, for introspection only.
/// This does not authorize execution.
pub fn required_capability(action: &AgentAction) -> Option<Capability> {
    Capability::for_action(action.action_type)
}

/// Convert capabilities from an already trusted policy/profile source into
/// kernel execution grants. Unknown and approval-only protocol values are
/// intentionally not grantable.
pub fn trusted_execution_grants(capabilities: &[Capability]) -> Vec<GrantedCapability> {
    capabilities
        .iter()
        .filter_map(|capability| match capability {
            Capability::ReadFile => Some(GrantedCapability::ReadFile),
            Capability::WriteFile => Some(GrantedCapability::WriteFile),
            Capability::PatchFile => Some(GrantedCapability::PatchFile),
            Capability::Execute => Some(GrantedCapability::Execute),
            Capability::Git => Some(GrantedCapability::Git),
            Capability::GitWrite => Some(GrantedCapability::GitWrite),
            Capability::GitDestructive => Some(GrantedCapability::GitDestructive),
            Capability::Mcp => Some(GrantedCapability::Mcp),
            Capability::RunTest => Some(GrantedCapability::RunTest),
            Capability::RequestApproval => Some(GrantedCapability::RequestApproval),
            Capability::ApprovalCritical | Capability::Unknown(_) => None,
        })
        .collect()
}

pub fn evaluate_policy(action: &AgentAction) -> Result<PolicyDecision, ExecutionError> {
    validate_action(action)?;
    Ok(match action.risk {
        RiskLevel::Low => PolicyDecision::Allow,
        RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical => {
            PolicyDecision::RequireApproval
        }
    })
}

/// Resolve risk policy against trusted approval evidence.
pub fn enforce_policy(
    action: &AgentAction,
    authority: &ExecutionAuthority,
) -> Result<PolicyDecision, ExecutionError> {
    match evaluate_policy(action)? {
        PolicyDecision::Allow => Ok(PolicyDecision::Allow),
        PolicyDecision::RequireApproval if authority.is_approved() => {
            Ok(PolicyDecision::RequireApproval)
        }
        PolicyDecision::RequireApproval => Err(ExecutionError::RequiresApproval),
        PolicyDecision::Deny => Err(ExecutionError::CapabilityDenied),
    }
}
