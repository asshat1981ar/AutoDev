use std::fs;

#[path = "../src/public_protocol.rs"]
mod public_protocol;

use public_protocol::PublicObjectiveEvent;

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
    let event: PublicObjectiveEvent = serde_json::from_str(&encoded).expect("typed objective event");

    assert_eq!(event.schema_version, "1");
    assert_eq!(event.event_type, "objective_queued");
    assert_eq!(event.objective_id, "obj-0001");
    assert_eq!(event.run_id, None);
    assert_eq!(event.task_id, None);

    let round_trip = serde_json::to_value(event).expect("serialize objective event");
    assert_eq!(round_trip["type"], "objective_queued");
    assert_eq!(round_trip["schema_version"], "1");
}
