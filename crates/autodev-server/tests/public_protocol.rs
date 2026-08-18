use std::fs;

pub struct ObjectiveRecord {
    pub id: String,
    pub repository: String,
    pub description: String,
    pub branch: String,
    pub status: String,
}

#[path = "../src/public_protocol.rs"]
mod public_protocol;

use public_protocol::{
    PublicObjectiveCreate, PublicObjectiveEvent, PublicObjectiveSummary, PublicProtocolError,
    PUBLIC_SCHEMA_VERSION,
};

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../protocols/public/v1/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    fs::read_to_string(path).expect("public protocol fixture")
}

#[test]
fn queued_objective_event_matches_canonical_fixture() {
    let encoded = fixture("objective-event.queued.json");
    let event: PublicObjectiveEvent =
        serde_json::from_str(&encoded).expect("typed objective event");

    assert_eq!(event.schema_version, PUBLIC_SCHEMA_VERSION);
    assert_eq!(event.event_type, "objective_queued");
    assert_eq!(event.objective_id, "obj-0001");
    assert_eq!(event.run_id, None);
    assert_eq!(event.task_id, None);

    let round_trip = serde_json::to_value(event).expect("serialize objective event");
    assert_eq!(round_trip["type"], "objective_queued");
    assert_eq!(round_trip["schema_version"], "1");
}

#[test]
fn queued_constructor_projects_objective_without_authority() {
    let record = ObjectiveRecord {
        id: "obj-generated".to_string(),
        repository: "owner/repo".to_string(),
        description: "Implement endpoint".to_string(),
        branch: "autodev/objective-generated".to_string(),
        status: "queued".to_string(),
    };

    let event = PublicObjectiveEvent::queued(&record);
    assert_eq!(event.schema_version, PUBLIC_SCHEMA_VERSION);
    assert_eq!(event.event_type, "objective_queued");
    assert_eq!(event.objective_id, record.id);
    assert_eq!(event.run_id, None);
    assert_eq!(event.task_id, None);
    assert!(!event.event_id.trim().is_empty());
    assert!(event.timestamp.ends_with('Z'));
    assert_eq!(event.data["repository"], record.repository);
    assert_eq!(event.data["branch"], record.branch);
    assert_eq!(event.data["status"], record.status);
    assert!(event.data.get("description").is_none());
    assert!(event.data.get("approval_ref").is_none());
}

#[test]
fn objective_summary_fixture_is_authority_safe() {
    let encoded = fixture("objective-summary.queued.json");
    let summary: PublicObjectiveSummary =
        serde_json::from_str(&encoded).expect("typed objective summary");

    assert_eq!(summary.schema_version, PUBLIC_SCHEMA_VERSION);
    assert_eq!(summary.id, "obj-0001");
    assert_eq!(summary.status, "queued");

    let value = serde_json::to_value(summary).expect("serialize objective summary");
    assert!(value.get("graph").is_none());
    assert!(value.get("capabilities").is_none());
    assert!(value.get("approval_ref").is_none());
}

#[test]
fn objective_create_fixture_is_untrusted_intent_only() {
    let encoded = fixture("objective-create.json");
    let create: PublicObjectiveCreate =
        serde_json::from_str(&encoded).expect("typed objective create");

    assert_eq!(create.repository, "owner/repo");
    assert_eq!(create.description, "Implement health endpoint");
    assert_eq!(create.branch.as_deref(), Some("autodev/objective-obj0001"));
}

#[test]
fn protocol_error_fixture_round_trips() {
    let encoded = fixture("protocol-error.json");
    let error: PublicProtocolError = serde_json::from_str(&encoded).expect("typed protocol error");

    assert_eq!(error.schema_version, PUBLIC_SCHEMA_VERSION);
    assert_eq!(error.code, "invalid_request");
    assert!(!error.retryable);
    assert_eq!(error.correlation_id, None);
}
