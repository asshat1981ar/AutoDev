use std::collections::VecDeque;

use forge_core::{
    ActionType, CodexEventTransport, CodexProposalClient, CodexProposalRequest, CodexRpcTransport,
    CodexSubscriptionClient, CodexSubscriptionError, RiskLevel,
};
use serde_json::{json, Value};

#[derive(Default)]
struct FakeEventTransport {
    calls: Vec<(String, Value)>,
    responses: VecDeque<Value>,
    notifications: VecDeque<Value>,
}

impl FakeEventTransport {
    fn new(
        responses: impl IntoIterator<Item = Value>,
        notifications: impl IntoIterator<Item = Value>,
    ) -> Self {
        Self {
            calls: Vec::new(),
            responses: responses.into_iter().collect(),
            notifications: notifications.into_iter().collect(),
        }
    }
}

impl CodexRpcTransport for FakeEventTransport {
    fn request(&mut self, method: &str, params: Value) -> Result<Value, CodexSubscriptionError> {
        self.calls.push((method.to_string(), params));
        self.responses
            .pop_front()
            .ok_or_else(|| CodexSubscriptionError::Protocol("missing fake response".into()))
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), CodexSubscriptionError> {
        self.calls.push((method.to_string(), params));
        Ok(())
    }
}

impl CodexEventTransport for FakeEventTransport {
    fn next_notification(&mut self) -> Result<Value, CodexSubscriptionError> {
        self.notifications
            .pop_front()
            .ok_or_else(|| CodexSubscriptionError::Protocol("missing fake notification".into()))
    }
}

fn proposal_action(task_id: &str, agent_id: &str) -> Value {
    json!({
        "id": "proposal-1",
        "task_id": task_id,
        "agent_id": agent_id,
        "type": "write_file",
        "reason": "update the bounded target",
        "risk": "medium",
        "capabilities": ["write_file"],
        "payload": {
            "path": "src/lib.rs",
            "content": "proposal only"
        },
        "expected": {
            "changed": true
        }
    })
}

#[test]
fn proposal_session_is_read_only_and_returns_typed_intent() {
    let action = proposal_action("task-1", "agent-1");
    let transport = FakeEventTransport::new(
        [
            json!({
                "account": {
                    "type": "chatgpt",
                    "planType": "plus"
                },
                "requiresOpenaiAuth": true
            }),
            json!({
                "thread": {"id": "thread-1"},
                "model": "gpt-5.3-codex"
            }),
            json!({
                "turn": {"id": "turn-1", "status": "inProgress"}
            }),
        ],
        [
            json!({
                "method": "item/completed",
                "params": {
                    "threadId": "thread-1",
                    "turnId": "turn-1",
                    "item": {
                        "type": "agentMessage",
                        "id": "item-1",
                        "text": action.to_string()
                    }
                }
            }),
            json!({
                "method": "turn/completed",
                "params": {
                    "threadId": "thread-1",
                    "turn": {
                        "id": "turn-1",
                        "status": "completed"
                    }
                }
            }),
        ],
    );
    let subscription = CodexSubscriptionClient::new_initialized_for_test(transport);
    let mut client =
        CodexProposalClient::from_authenticated_subscription(subscription, "/isolated")
            .expect("authenticated subscription is accepted");

    let proposed = client
        .propose_action(&CodexProposalRequest {
            model: "gpt-5.3-codex".into(),
            task_id: "task-1".into(),
            agent_id: "agent-1".into(),
            prompt: "Propose the smallest safe code change.".into(),
        })
        .expect("proposal succeeds");

    assert_eq!(proposed.action_type, ActionType::WriteFile);
    assert_eq!(proposed.risk, RiskLevel::Medium);
    assert_eq!(proposed.task_id, "task-1");
    assert_eq!(proposed.agent_id, "agent-1");

    let calls = &client.transport().calls;
    assert_eq!(calls[0], ("account/read".into(), json!({})));
    assert_eq!(calls[1].0, "thread/start");
    assert_eq!(calls[1].1["model"], "gpt-5.3-codex");
    assert_eq!(calls[1].1["sandbox"], "read-only");
    assert_eq!(calls[1].1["approvalPolicy"], "never");
    assert_eq!(calls[1].1["ephemeral"], true);
    assert_eq!(calls[1].1["cwd"], "/isolated");
    assert!(calls[1].1["developerInstructions"]
        .as_str()
        .expect("developer instructions")
        .contains("ForgeCore"));

    assert_eq!(calls[2].0, "turn/start");
    assert_eq!(calls[2].1["threadId"], "thread-1");
    assert_eq!(calls[2].1["input"][0]["type"], "text");
    assert_eq!(
        calls[2].1["input"][0]["text"],
        "Propose the smallest safe code change."
    );
    assert!(calls[2].1["outputSchema"].is_object());
}

#[test]
fn proposal_rejects_model_identity_drift() {
    let action = proposal_action("other-task", "agent-1");
    let transport = FakeEventTransport::new(
        [
            json!({
                "account": {
                    "type": "chatgpt",
                    "planType": "plus"
                },
                "requiresOpenaiAuth": true
            }),
            json!({"thread": {"id": "thread-1"}, "model": "gpt-5.3-codex"}),
            json!({"turn": {"id": "turn-1", "status": "inProgress"}}),
        ],
        [
            json!({
                "method": "item/completed",
                "params": {
                    "item": {
                        "type": "agentMessage",
                        "id": "item-1",
                        "text": action.to_string()
                    }
                }
            }),
            json!({
                "method": "turn/completed",
                "params": {
                    "turn": {"id": "turn-1", "status": "completed"}
                }
            }),
        ],
    );
    let subscription = CodexSubscriptionClient::new_initialized_for_test(transport);
    let mut client =
        CodexProposalClient::from_authenticated_subscription(subscription, "/isolated")
            .expect("authenticated subscription is accepted");

    let error = client
        .propose_action(&CodexProposalRequest {
            model: "gpt-5.3-codex".into(),
            task_id: "task-1".into(),
            agent_id: "agent-1".into(),
            prompt: "Propose a change.".into(),
        })
        .expect_err("identity drift must fail closed");

    assert!(matches!(error, CodexSubscriptionError::Protocol(_)));
}
