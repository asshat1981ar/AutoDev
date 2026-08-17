use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveStatus {
    Queued,
    Planning,
    Running,
    Blocked,
    Verifying,
    Replanned,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveView {
    pub id: String,
    pub repository: String,
    pub description: String,
    pub branch: String,
    pub status: ObjectiveStatus,
    pub current_task_id: Option<String>,
    pub current_phase: Option<String>,
    pub latest_evidence_ref: Option<String>,
    pub blocked_reason: Option<String>,
}
