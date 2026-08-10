//! ForgeCore is the trusted execution boundary for AutoDev.
//!
//! Agents produce intent. ForgeCore will eventually execute only intent that
//! has passed policy evaluation. The initial implementation deliberately
//! contains no privileged filesystem or process execution.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    ReadFile,
    WriteFile,
    PatchFile,
    Execute,
    Git,
    Mcp,
    RunTest,
    RequestApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentAction {
    pub id: String,
    pub task_id: String,
    pub agent_id: String,
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub reason: String,
    pub risk: RiskLevel,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    RequireApproval,
    Deny,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolicyError {
    #[error("action id must not be empty")]
    MissingActionId,
    #[error("task id must not be empty")]
    MissingTaskId,
    #[error("agent id must not be empty")]
    MissingAgentId,
    #[error("action reason must not be empty")]
    MissingReason,
    #[error("critical actions require explicit approval capability")]
    CriticalApprovalRequired,
}

/// Validate the structural invariants that must hold before policy evaluation.
pub fn validate_action(action: &AgentAction) -> Result<(), PolicyError> {
    if action.id.trim().is_empty() {
        return Err(PolicyError::MissingActionId);
    }
    if action.task_id.trim().is_empty() {
        return Err(PolicyError::MissingTaskId);
    }
    if action.agent_id.trim().is_empty() {
        return Err(PolicyError::MissingAgentId);
    }
    if action.reason.trim().is_empty() {
        return Err(PolicyError::MissingReason);
    }
    if action.risk == RiskLevel::Critical
        && !action.capabilities.iter().any(|c| c == "approval:critical")
    {
        return Err(PolicyError::CriticalApprovalRequired);
    }
    Ok(())
}

/// Conservative initial policy. Real capability and workspace policy will be
/// introduced after this contract is covered by tests.
pub fn evaluate_policy(action: &AgentAction) -> Result<PolicyDecision, PolicyError> {
    validate_action(action)?;

    Ok(match action.risk {
        RiskLevel::Low => PolicyDecision::Allow,
        RiskLevel::Medium | RiskLevel::High => PolicyDecision::RequireApproval,
        RiskLevel::Critical => PolicyDecision::RequireApproval,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Accepted,
    Denied,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionResult {
    pub action_id: String,
    pub status: ExecutionStatus,
    pub message: String,
}

/// Dry-run executor. It intentionally refuses to perform privileged effects.
pub fn dry_run(action: &AgentAction) -> Result<ExecutionResult, PolicyError> {
    let decision = evaluate_policy(action)?;

    match decision {
        PolicyDecision::Allow => Ok(ExecutionResult {
            action_id: action.id.clone(),
            status: ExecutionStatus::Accepted,
            message: "action authorized; execution adapter not enabled".into(),
        }),
        PolicyDecision::RequireApproval => Ok(ExecutionResult {
            action_id: action.id.clone(),
            status: ExecutionStatus::Denied,
            message: "action requires approval before execution".into(),
        }),
        PolicyDecision::Deny => Ok(ExecutionResult {
            action_id: action.id.clone(),
            status: ExecutionStatus::Denied,
            message: "action denied by policy".into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(risk: RiskLevel) -> AgentAction {
        AgentAction {
            id: "action-1".into(),
            task_id: "task-1".into(),
            agent_id: "agent-1".into(),
            action_type: ActionType::ReadFile,
            reason: "inspect source".into(),
            risk,
            capabilities: vec![],
            payload: serde_json::json!({"path": "README.md"}),
        }
    }

    #[test]
    fn low_risk_actions_are_allowed() {
        assert_eq!(evaluate_policy(&action(RiskLevel::Low)).unwrap(), PolicyDecision::Allow);
    }

    #[test]
    fn medium_risk_actions_require_approval() {
        assert_eq!(
            evaluate_policy(&action(RiskLevel::Medium)).unwrap(),
            PolicyDecision::RequireApproval
        );
    }

    #[test]
    fn critical_actions_require_explicit_approval_capability() {
        let error = evaluate_policy(&action(RiskLevel::Critical)).unwrap_err();
        assert_eq!(error, PolicyError::CriticalApprovalRequired);
    }

    #[test]
    fn empty_identity_is_rejected() {
        let mut candidate = action(RiskLevel::Low);
        candidate.agent_id.clear();
        assert_eq!(validate_action(&candidate).unwrap_err(), PolicyError::MissingAgentId);
    }

    #[test]
    fn dry_run_never_executes_privileged_operations() {
        let result = dry_run(&action(RiskLevel::Low)).unwrap();
        assert_eq!(result.status, ExecutionStatus::Accepted);
        assert!(result.message.contains("execution adapter not enabled"));
    }
}
