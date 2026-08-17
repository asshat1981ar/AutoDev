use serde::Serialize;

use crate::{ObjectiveStatus, ObjectiveView};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObjectiveEvent {
    #[serde(rename = "type")]
    pub event_type: &'static str,
    pub objective_id: String,
    pub task_id: Option<String>,
    pub phase: Option<String>,
    pub status: ObjectiveStatus,
    pub evidence_ref: Option<String>,
    pub message: String,
}

impl ObjectiveEvent {
    pub fn from_view(view: &ObjectiveView, message: impl Into<String>) -> Self {
        let event_type = match view.status {
            ObjectiveStatus::Queued => "objective_queued",
            ObjectiveStatus::Planning => "objective_planning",
            ObjectiveStatus::Running => "objective_running",
            ObjectiveStatus::Blocked => "objective_blocked",
            ObjectiveStatus::Verifying => "objective_verifying",
            ObjectiveStatus::Replanned => "objective_replanned",
            ObjectiveStatus::Completed => "objective_completed",
            ObjectiveStatus::Failed => "objective_failed",
        };
        Self {
            event_type,
            objective_id: view.id.clone(),
            task_id: view.current_task_id.clone(),
            phase: view.current_phase.clone(),
            status: view.status,
            evidence_ref: view.latest_evidence_ref.clone(),
            message: message.into(),
        }
    }
}
