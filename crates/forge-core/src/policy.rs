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

/// Kernel-owned capability authority used by execution adapters.
///
/// This type deliberately does not implement `Serialize` or `Deserialize` and
/// its granting constructor is crate-private. Model-generated protocol objects
/// can request capabilities, but only trusted ForgeCore code can mint effective
/// execution authority.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionAuthority {
    capabilities: Vec<Capability>,
}

impl ExecutionAuthority {
    /// Construct an authority that grants nothing. This is the safe default for
    /// public entry points that have no trusted authorization context.
    pub fn deny_all() -> Self {
        Self::default()
    }

    /// Mint effective authority from capability state already established by a
    /// trusted kernel path. This is intentionally unavailable outside ForgeCore.
    pub(crate) fn from_trusted_capabilities(
        capabilities: impl IntoIterator<Item = Capability>,
    ) -> Self {
        let mut effective = Vec::new();
        for capability in capabilities {
            if !effective.iter().any(|existing| existing == &capability) {
                effective.push(capability);
            }
        }
        Self {
            capabilities: effective,
        }
    }

    /// Whether this authority grants a concrete capability.
    pub fn allows(&self, capability: &Capability) -> bool {
        self.capabilities
            .iter()
            .any(|granted| granted == capability)
    }
}

/// Kernel-owned approval material. Unlike `AgentAction` fields, this value is
/// never supplied by the model; it is created by the trusted orchestration
/// boundary after an approval decision has been recorded.
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

/// Legacy intent-level capability check.
///
/// This helper remains temporarily for non-migrated call sites. Execution
/// adapters must use [`has_required_execution_authority`] instead. The values on
/// `AgentAction` are requests/diagnostics and are not effective authority.
pub fn has_required_capability(action: &AgentAction) -> bool {
    match Capability::for_action(action.action_type) {
        Some(required) => action.capabilities.iter().any(|c| c == &required),
        None => true,
    }
}

/// Check the capability required by an action against kernel-owned authority.
pub fn has_required_execution_authority(
    action: &AgentAction,
    authority: &ExecutionAuthority,
) -> bool {
    match Capability::for_action(action.action_type) {
        Some(required) => authority.allows(&required),
        None => true,
    }
}

/// The capability required by an action, if any.
pub fn required_capability(action: &AgentAction) -> Option<Capability> {
    Capability::for_action(action.action_type)
}

/// Evaluate the full policy for an action.
///
/// Performs structural validation, then applies risk-based authorization.
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
/// An agent cannot satisfy approval by changing its payload or requested
/// capabilities; only a kernel-created `AuthorizationGrant` can discharge a
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
