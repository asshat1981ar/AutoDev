//! Proposal-only Codex adapter.
//!
//! Codex generation is intentionally separated from ForgeCore execution. This
//! adapter accepts a ChatGPT-subscription-backed app-server connection and
//! returns a typed [`AgentAction`] as untrusted intent. It never authorizes or
//! executes the proposed action.

use serde_json::{json, Value};

use crate::{AgentAction, CodexEventTransport, CodexSubscriptionClient, CodexSubscriptionError};

const PROPOSAL_ONLY_INSTRUCTIONS: &str = "You are AutoDev's proposal-only Codex component. Return exactly one AutoDev AgentAction JSON object matching the supplied output schema. Do not execute commands, mutate files, call tools, request approvals, or claim authorization. ForgeCore alone authorizes and executes proposed actions.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexProposalRequest {
    pub model: String,
    pub task_id: String,
    pub agent_id: String,
    pub prompt: String,
}

pub struct CodexProposalClient<T: CodexEventTransport> {
    transport: T,
    proposal_cwd: String,
}

impl<T: CodexEventTransport> CodexProposalClient<T> {
    pub fn from_authenticated_subscription(
        mut subscription: CodexSubscriptionClient<T>,
        proposal_cwd: impl Into<String>,
    ) -> Result<Self, CodexSubscriptionError> {
        let account = subscription.account()?;
        if !account.authenticated || account.auth_mode.as_deref() != Some("chatgpt") {
            return Err(CodexSubscriptionError::Protocol(
                "Codex proposal provider requires an authenticated ChatGPT subscription".into(),
            ));
        }

        let proposal_cwd = proposal_cwd.into();
        if proposal_cwd.trim().is_empty() {
            return Err(CodexSubscriptionError::Protocol(
                "proposal working directory is required".into(),
            ));
        }

        Ok(Self {
            transport: subscription.into_transport(),
            proposal_cwd,
        })
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn propose_action(
        &mut self,
        request: &CodexProposalRequest,
    ) -> Result<AgentAction, CodexSubscriptionError> {
        validate_request(request)?;

        let thread = self.transport.request(
            "thread/start",
            json!({
                "model": request.model,
                "cwd": self.proposal_cwd,
                "approvalPolicy": "never",
                "sandbox": "read-only",
                "ephemeral": true,
                "developerInstructions": PROPOSAL_ONLY_INSTRUCTIONS,
            }),
        )?;
        let thread_id = required_nested_string(&thread, &["thread", "id"])?;
        if let Some(actual_model) = thread.get("model").and_then(Value::as_str) {
            if actual_model != request.model {
                return Err(CodexSubscriptionError::Protocol(format!(
                    "Codex model drift: requested {}, received {actual_model}",
                    request.model
                )));
            }
        }

        let turn = self.transport.request(
            "turn/start",
            json!({
                "threadId": thread_id,
                "input": [{
                    "type": "text",
                    "text": request.prompt,
                    "textElements": [],
                }],
                "outputSchema": agent_action_output_schema(),
            }),
        )?;
        let turn_id = required_nested_string(&turn, &["turn", "id"])?;

        let final_message = self.collect_final_message(&thread_id, &turn_id)?;
        let action: AgentAction = serde_json::from_str(&final_message).map_err(|error| {
            CodexSubscriptionError::Protocol(format!("invalid AgentAction proposal: {error}"))
        })?;

        if action.task_id != request.task_id {
            return Err(CodexSubscriptionError::Protocol(format!(
                "proposal task identity drift: expected {}, received {}",
                request.task_id, action.task_id
            )));
        }
        if action.agent_id != request.agent_id {
            return Err(CodexSubscriptionError::Protocol(format!(
                "proposal agent identity drift: expected {}, received {}",
                request.agent_id, action.agent_id
            )));
        }

        Ok(action)
    }

    fn collect_final_message(
        &mut self,
        thread_id: &str,
        turn_id: &str,
    ) -> Result<String, CodexSubscriptionError> {
        let mut final_message = None;

        loop {
            let notification = self.transport.next_notification()?;
            let method = notification
                .get("method")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    CodexSubscriptionError::Protocol("Codex notification is missing method".into())
                })?;
            let params = notification.get("params").ok_or_else(|| {
                CodexSubscriptionError::Protocol("Codex notification is missing params".into())
            })?;
            ensure_identity_if_present(params, "threadId", thread_id)?;
            ensure_identity_if_present(params, "turnId", turn_id)?;

            match method {
                "item/completed" => {
                    let item = params.get("item").ok_or_else(|| {
                        CodexSubscriptionError::Protocol(
                            "item/completed notification is missing item".into(),
                        )
                    })?;
                    match item.get("type").and_then(Value::as_str) {
                        Some("agentMessage") => {
                            final_message = Some(
                                item.get("text")
                                    .and_then(Value::as_str)
                                    .filter(|text| !text.trim().is_empty())
                                    .ok_or_else(|| {
                                        CodexSubscriptionError::Protocol(
                                            "completed agent message is empty".into(),
                                        )
                                    })?
                                    .to_string(),
                            );
                        }
                        Some(
                            "commandExecution"
                            | "fileChange"
                            | "mcpToolCall"
                            | "dynamicToolCall"
                            | "collabAgentToolCall",
                        ) => {
                            return Err(CodexSubscriptionError::Protocol(
                                "proposal-only Codex session attempted a side-effect item".into(),
                            ));
                        }
                        _ => {}
                    }
                }
                "turn/completed" => {
                    let status = params
                        .get("turn")
                        .and_then(|turn| turn.get("status"))
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            CodexSubscriptionError::Protocol(
                                "turn/completed notification is missing status".into(),
                            )
                        })?;
                    if status != "completed" {
                        return Err(CodexSubscriptionError::Protocol(format!(
                            "Codex proposal turn ended with status {status}"
                        )));
                    }
                    break;
                }
                _ => {}
            }
        }

        final_message.ok_or_else(|| {
            CodexSubscriptionError::Protocol(
                "Codex proposal turn completed without an agent message".into(),
            )
        })
    }
}

fn validate_request(request: &CodexProposalRequest) -> Result<(), CodexSubscriptionError> {
    for (name, value) in [
        ("model", request.model.as_str()),
        ("task_id", request.task_id.as_str()),
        ("agent_id", request.agent_id.as_str()),
        ("prompt", request.prompt.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(CodexSubscriptionError::Protocol(format!(
                "Codex proposal {name} is required"
            )));
        }
    }
    Ok(())
}

fn required_nested_string(value: &Value, path: &[&str]) -> Result<String, CodexSubscriptionError> {
    let mut current = value;
    for key in path {
        current = current.get(*key).ok_or_else(|| {
            CodexSubscriptionError::Protocol(format!("missing {}", path.join(".")))
        })?;
    }
    current
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| CodexSubscriptionError::Protocol(format!("invalid {}", path.join("."))))
}

fn ensure_identity_if_present(
    params: &Value,
    field: &str,
    expected: &str,
) -> Result<(), CodexSubscriptionError> {
    if let Some(actual) = params.get(field).and_then(Value::as_str) {
        if actual != expected {
            return Err(CodexSubscriptionError::Protocol(format!(
                "Codex notification {field} drift: expected {expected}, received {actual}"
            )));
        }
    }
    Ok(())
}

fn agent_action_output_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": [
            "id",
            "task_id",
            "agent_id",
            "type",
            "reason",
            "risk",
            "requested_capabilities",
            "payload",
            "expected"
        ],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "task_id": {"type": "string", "minLength": 1},
            "agent_id": {"type": "string", "minLength": 1},
            "type": {
                "type": "string",
                "enum": [
                    "read_file",
                    "write_file",
                    "patch_file",
                    "execute",
                    "git",
                    "mcp",
                    "run_test",
                    "request_approval"
                ]
            },
            "reason": {"type": "string"},
            "risk": {
                "type": "string",
                "enum": ["low", "medium", "high", "critical"]
            },
            "requested_capabilities": {
                "type": "array",
                "items": {"type": "string"}
            },
            "payload": {"type": "object"},
            "expected": {"type": "object"}
        }
    })
}
