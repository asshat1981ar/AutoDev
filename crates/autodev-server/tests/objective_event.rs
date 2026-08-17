use autodev_server::{ObjectiveEvent, ObjectiveStatus, ObjectiveView};
use serde_json::{json, Value};

fn view(status: ObjectiveStatus) -> ObjectiveView {
    ObjectiveView {
        id: "objective-1".to_string(),
        repository: "asshat1981ar/AutoDev".to_string(),
        description: "Emit lifecycle events".to_string(),
        branch: "autodev/events".to_string(),
        status,
        current_task_id: Some("t-root".to_string()),
        current_phase: Some("plan".to_string()),
        latest_evidence_ref: None,
        blocked_reason: None,
    }
}

#[test]
fn objective_event_type_is_derived_from_closed_lifecycle_status() {
    let cases = [
        (ObjectiveStatus::Queued, "objective_queued"),
        (ObjectiveStatus::Planning, "objective_planning"),
        (ObjectiveStatus::Running, "objective_running"),
        (ObjectiveStatus::Blocked, "objective_blocked"),
        (ObjectiveStatus::Verifying, "objective_verifying"),
        (ObjectiveStatus::Replanned, "objective_replanned"),
        (ObjectiveStatus::Completed, "objective_completed"),
        (ObjectiveStatus::Failed, "objective_failed"),
    ];

    for (status, event_type) in cases {
        let event = ObjectiveEvent::from_view(&view(status), "lifecycle transition");
        assert_eq!(event.event_type, event_type);
    }
}

#[test]
fn queued_event_serializes_as_flat_safe_payload() {
    let event = ObjectiveEvent::from_view(&view(ObjectiveStatus::Queued), "objective accepted");
    let value = serde_json::to_value(event).expect("serialize event");

    assert_eq!(
        value,
        json!({
            "type": "objective_queued",
            "objective_id": "objective-1",
            "task_id": "t-root",
            "phase": "plan",
            "status": "queued",
            "evidence_ref": Value::Null,
            "message": "objective accepted"
        })
    );
    assert!(value.get("approval_ref").is_none());
    assert!(value.get("data").is_none());
}
