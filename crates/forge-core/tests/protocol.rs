//! Integration tests for the agent-action protocol wire contract.
//!
//! These assert that the Rust types serialize to and deserialize from the JSON
//! shapes defined in `protocols/*.schema.json`.

use forge_core::{
    ActionType, AgentAction, Capability, ExecutionResult, ExecutionStatus, RiskLevel, TaskNode,
    TaskStatus,
};
use serde_json::json;
use std::path::PathBuf;

fn schema(name: &str) -> serde_json::Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../protocols")
        .join(name);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn assert_object_matches_schema(value: &serde_json::Value, schema: &serde_json::Value) {
    let object = value.as_object().unwrap();
    let properties = schema["properties"].as_object().unwrap();
    for field in object.keys() {
        assert!(
            properties.contains_key(field),
            "schema is missing field {field}"
        );
    }
    for field in schema["required"].as_array().unwrap() {
        let field = field.as_str().unwrap();
        assert!(
            object.contains_key(field),
            "serialized value is missing required field {field}"
        );
    }
}

fn assert_enum_values(schema: &serde_json::Value, property: &str, values: &[&str]) {
    let allowed: Vec<&str> = schema["properties"][property]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(allowed, values);
}

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
    // Capability intent is explicit on the wire and is not an authorization grant.
    assert_eq!(obj["requested_capabilities"], json!(["read_file"]));
    assert!(!obj.contains_key("capabilities"));
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
    for capability in [
        Capability::ReadFile,
        Capability::WriteFile,
        Capability::PatchFile,
        Capability::Execute,
        Capability::Git,
        Capability::GitWrite,
        Capability::GitDestructive,
        Capability::Mcp,
        Capability::RunTest,
        Capability::ApprovalCritical,
        Capability::Other("custom:tool".to_string()),
    ] {
        let json = serde_json::to_value(&capability).unwrap();
        let restored: Capability = serde_json::from_value(json).unwrap();
        assert_eq!(restored, capability);
    }
}

#[test]
fn unknown_capability_is_preserved() {
    let capability: Capability = serde_json::from_value(json!("plugin:custom")).unwrap();
    assert_eq!(capability, Capability::Other("plugin:custom".to_string()));
}

#[test]
fn action_schema_tracks_rust_wire_contract() {
    let schema = schema("agent-action.schema.json");
    assert_enum_values(
        &schema,
        "type",
        &[
            "read_file",
            "write_file",
            "patch_file",
            "execute",
            "git",
            "mcp",
            "run_test",
            "request_approval",
        ],
    );
    assert_enum_values(&schema, "risk", &["low", "medium", "high", "critical"]);
    assert_object_matches_schema(&serde_json::to_value(sample_action()).unwrap(), &schema);
}

#[test]
fn execution_result_schema_tracks_rust_wire_contract() {
    let result = ExecutionResult {
        action_id: "action-1".to_string(),
        status: ExecutionStatus::Succeeded,
        started_at: chrono::Utc::now(),
        completed_at: chrono::Utc::now(),
        exit_code: Some(0),
        stdout: "ok".to_string(),
        stderr: String::new(),
        artifacts: vec!["artifact".to_string()],
        verification: Some(json!({"passed": true})),
        error: None,
    };
    let schema = schema("execution-result.schema.json");
    assert_enum_values(
        &schema,
        "status",
        &[
            "accepted",
            "denied",
            "running",
            "succeeded",
            "failed",
            "cancelled",
        ],
    );
    assert_object_matches_schema(&serde_json::to_value(result).unwrap(), &schema);
}

#[test]
fn task_schema_tracks_rust_wire_contract() {
    let task = TaskNode::new("task-1", "title", "description");
    let schema = schema("task.schema.json");
    assert_enum_values(
        &schema,
        "status",
        &[
            "queued",
            "planning",
            "ready",
            "running",
            "verifying",
            "repairing",
            "blocked",
            "completed",
            "failed",
            "cancelled",
        ],
    );
    assert_object_matches_schema(&serde_json::to_value(task).unwrap(), &schema);
}
