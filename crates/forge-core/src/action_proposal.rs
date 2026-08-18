use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

use crate::policy::effective_risk_for_action;
use crate::{
    ActionType, AgentAction, AgentProfile, Capability, ModelError, ModelProvider, ModelRequest,
    PolicyDecision, RiskLevel, Task,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ActionProposal {
    pub action: AgentAction,
    pub decision: PolicyDecision,
    pub model: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelAction {
    action: String,
    reason: String,
    risk: RiskLevel,
    #[serde(default)]
    payload: Value,
}

pub fn propose_action(
    agent_id: &str,
    profile: &AgentProfile,
    provider: &dyn ModelProvider,
    task: &Task,
) -> Result<ActionProposal, ActionProposalError> {
    let request = ModelRequest {
        model: profile.model.preferred.clone(),
        messages: None,
        prompt: Some(format!(
            "{}\n\nTask: {}\nContext: {}",
            constrained_prompt(profile),
            task.title,
            task.context,
        )),
        options: None,
    };
    let response = provider.generate(&request)?;
    let parsed: ModelAction = serde_json::from_str(response.content.trim())?;
    let action_type = parse_action_type(&parsed.action)?;
    let mut action = AgentAction {
        id: format!("{}:{}", task.id, parsed.action),
        task_id: task.id.clone(),
        agent_id: agent_id.to_string(),
        action_type,
        reason: parsed.reason,
        risk: parsed.risk,
        capabilities: vec![],
        payload: parsed.payload,
        expected: Value::Null,
    };
    action.risk = effective_risk_for_action(&action);

    let tool_allowed = profile
        .policy
        .tools
        .iter()
        .any(|tool| tool == action.action_type.as_str());
    let decision = if !tool_allowed {
        PolicyDecision::Deny
    } else {
        match Capability::for_action(action.action_type) {
            Some(required) if !profile.may(&required, action.risk) => PolicyDecision::Deny,
            _ => match action.risk {
                RiskLevel::Low => PolicyDecision::Allow,
                RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical => {
                    PolicyDecision::RequireApproval
                }
            },
        }
    };

    Ok(ActionProposal {
        action,
        decision,
        model: response.model,
    })
}

fn constrained_prompt(profile: &AgentProfile) -> String {
    let capabilities = profile
        .capabilities
        .iter()
        .map(|capability| serde_json::to_string(capability).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(", ");
    let tools = profile.policy.tools.join(", ");
    format!(
        "Return one JSON object only with fields action, reason, risk, payload. \
         Do not include approval_ref, approved, trusted capabilities, authorization grants, or evidence claims. \
         Role: {}. Allowed tools: [{}]. Allowed capabilities: [{}]. Risk ceiling: {:?}.",
        profile.role.as_str(),
        tools,
        capabilities,
        profile.policy.risk_ceiling,
    )
}

fn parse_action_type(name: &str) -> Result<ActionType, ActionProposalError> {
    match name {
        "read_file" => Ok(ActionType::ReadFile),
        "write_file" => Ok(ActionType::WriteFile),
        "patch_file" => Ok(ActionType::PatchFile),
        "execute" => Ok(ActionType::Execute),
        "git" => Ok(ActionType::Git),
        "mcp" => Ok(ActionType::Mcp),
        "run_test" => Ok(ActionType::RunTest),
        "request_approval" => Ok(ActionType::RequestApproval),
        other => Err(ActionProposalError::UnsupportedAction(other.to_string())),
    }
}

#[derive(Debug, Error)]
pub enum ActionProposalError {
    #[error(transparent)]
    Provider(#[from] ModelError),
    #[error("model returned invalid action JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("model proposed unsupported action '{0}'")]
    UnsupportedAction(String),
}
