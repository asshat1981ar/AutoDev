use crate::{
    evaluate_policy, has_required_capability, ActionType, AgentAction, AgentProfile, Capability,
    ModelProvider, ModelRequest, ModelResponse, PolicyDecision, RiskLevel, RuntimeError,
    StructuredOutput, Task,
};

/// A model-generated action that has been structurally validated and evaluated
/// by policy, but has not been authorized or executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionProposal {
    pub action: AgentAction,
    pub decision: PolicyDecision,
    pub model: String,
}

/// Produce typed, policy-evaluated intent without invoking any executor or
/// creating trusted approval state.
pub fn propose_action(
    agent_id: &str,
    profile: &AgentProfile,
    provider: &dyn ModelProvider,
    task: &Task,
) -> Result<ActionProposal, RuntimeError> {
    let model = select_model(agent_id, profile, provider)?;
    propose_action_with_model(agent_id, profile, provider, task, &model)
}

pub(crate) fn propose_action_with_model(
    agent_id: &str,
    profile: &AgentProfile,
    provider: &dyn ModelProvider,
    task: &Task,
    model: &str,
) -> Result<ActionProposal, RuntimeError> {
    let response = invoke_model(provider, profile, Some(task), model)?;
    let action = validate_output(agent_id, profile, task, &response)?;
    let decision = submit_to_policy(&action)?;
    Ok(ActionProposal {
        action,
        decision,
        model: model.to_string(),
    })
}

pub(crate) fn assemble_context(profile: &AgentProfile, task: Option<&Task>) -> String {
    let task = task
        .map(|task| format!("Task: {} — {}", task.title, task.context))
        .unwrap_or_else(|| "No task".to_string());
    let capabilities: Vec<&str> = profile
        .capabilities
        .iter()
        .map(|capability| match capability {
            Capability::Unknown(value) => value.as_str(),
            _ => capability.as_str(),
        })
        .collect();
    format!(
        "You are the {role} agent.\n{capabilities:?}\n{task}\nReturn one structured action.",
        role = profile.role.as_str()
    )
}

pub(crate) fn select_model(
    agent_id: &str,
    profile: &AgentProfile,
    provider: &dyn ModelProvider,
) -> Result<String, RuntimeError> {
    let preferred = profile.model.preferred.clone();
    let models = provider
        .list_models()
        .map_err(|error| RuntimeError::NoModel(error.to_string()))?;
    if models.iter().any(|model| model.id == preferred) {
        return Ok(preferred);
    }
    models
        .iter()
        .find(|model| model.capabilities.chat)
        .map(|model| model.id.clone())
        .ok_or_else(|| RuntimeError::NoModel(agent_id.to_string()))
}

pub(crate) fn invoke_model(
    provider: &dyn ModelProvider,
    profile: &AgentProfile,
    task: Option<&Task>,
    model: &str,
) -> Result<ModelResponse, RuntimeError> {
    let request = ModelRequest {
        model: model.to_string(),
        messages: Some(vec![crate::model::Message {
            role: "user".to_string(),
            content: assemble_context(profile, task),
        }]),
        prompt: None,
        options: None,
    };
    provider
        .chat(&request)
        .map_err(|error| RuntimeError::ExecutionFailed(error.to_string()))
}

pub(crate) fn validate_output(
    agent_id: &str,
    profile: &AgentProfile,
    task: &Task,
    response: &ModelResponse,
) -> Result<AgentAction, RuntimeError> {
    let parsed: StructuredOutput = serde_json::from_str(&response.content)
        .map_err(|error| RuntimeError::InvalidOutput(error.to_string()))?;
    let action_type = parse_action_type(&parsed.action)
        .ok_or_else(|| RuntimeError::InvalidOutput("unknown action type".into()))?;
    let risk = parse_risk(&parsed.risk)
        .ok_or_else(|| RuntimeError::InvalidOutput("unknown risk".into()))?;
    Ok(AgentAction {
        id: format!("{agent_id}-{}", task.id),
        task_id: task.id.clone(),
        agent_id: agent_id.to_string(),
        action_type,
        reason: parsed.reason,
        risk,
        capabilities: profile.capabilities.clone(),
        payload: parsed.payload,
        expected: serde_json::json!({}),
    })
}

pub(crate) fn submit_to_policy(action: &AgentAction) -> Result<PolicyDecision, RuntimeError> {
    evaluate_policy(action).map_err(|error| RuntimeError::PolicyDenied(error.to_string()))?;
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

fn parse_action_type(value: &str) -> Option<ActionType> {
    Some(match value {
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

fn parse_risk(value: &str) -> Option<RiskLevel> {
    Some(match value {
        "low" => RiskLevel::Low,
        "medium" => RiskLevel::Medium,
        "high" => RiskLevel::High,
        "critical" => RiskLevel::Critical,
        _ => return None,
    })
}
