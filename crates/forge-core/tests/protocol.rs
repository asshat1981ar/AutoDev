//! Integration tests for the agent-action protocol wire contract.
//!
//! These assert that the Rust types serialize to and deserialize from the JSON
//! shapes defined in `protocols/*.schema.json`.

use forge_core::{ActionType, AgentAction, Capability, RiskLevel};
use serde_json::json;

fn sample_action() -> AgentAction {
    AgentAction {
        id: "action-1".to_string(),
        task_id: "task-1".to_string(),
        agent_id: "agent-1".to_string(),
        action_type: ActionType::ReadFile,
        reason: "inspect source".to_string(),
        risk: RiskLevel::Low,
        capabilities: vec![Capability::ReadFile],
        payload: json!({ "path": "README.md" }),
        expected: json!({}),
    }
}

#[test]
fn action_serializes_to_schema_shape() {
    let value = serde_json::to_value(sample_action()).unwrap();
    let obj = value.as_object().unwrap();

    // Field names match the schema.
    for field in [
        "id", "task_id", "agent_id", "type", "reason", "risk", "payload",
    ] {
        assert!(obj.contains_key(field), "missing field {field}");
    }
    // The `type` field is present and snake_case.
    assert_eq!(obj["type"], "read_file");
    assert_eq!(obj["risk"], "low");
    // `capabilities` defaults to an array.
    assert_eq!(obj["capabilities"], json!(["read_file"]));
}

#[test]
fn action_round_trips() {
    let json = serde_json::to_value(sample_action()).unwrap();
    let back: AgentAction = serde_json::from_value(json).unwrap();
    assert_eq!(back, sample_action());
}

#[test]
fn unknown_action_type_is_rejected() {
    let mut json = serde_json::to_value(sample_action()).unwrap();
    json["type"] = json!("delete_everything"); // not in the schema enum
    let err = serde_json::from_value::<AgentAction>(json).unwrap_err();
    assert!(err.is_data());
}

#[test]
fn unknown_extra_field_is_rejected() {
    let mut json = serde_json::to_value(sample_action()).unwrap();
    json["sneaky"] = json!(true);
    let res = serde_json::from_value::<AgentAction>(json);
    assert!(res.is_err());
}

#[test]
fn missing_required_field_is_rejected() {
    let mut json = serde_json::to_value(sample_action()).unwrap();
    json.as_object_mut().unwrap().remove("payload");
    let res = serde_json::from_value::<AgentAction>(json);
    assert!(res.is_err());
}

#[test]
fn malformed_risk_is_rejected() {
    let mut json = serde_json::to_value(sample_action()).unwrap();
    json["risk"] = json!("critical!!");
    let res = serde_json::from_value::<AgentAction>(json);
    assert!(res.is_err());
}

#[test]
fn capability_round_trips() {
    assert_eq!(
        serde_json::to_value(Capability::ReadFile).unwrap(),
        json!("read_file")
    );
    assert_eq!(
        serde_json::to_value(Capability::ApprovalCritical).unwrap(),
        json!("approval:critical")
    );
    assert_eq!(
        serde_json::from_value::<Capability>(json!("git")).unwrap(),
        Capability::Git
    );
}

#[test]
fn unknown_capability_is_preserved() {
    let cap: Capability = serde_json::from_value(json!("future:thing")).unwrap();
    assert_eq!(cap, Capability::Unknown("future:thing".to_string()));
}
