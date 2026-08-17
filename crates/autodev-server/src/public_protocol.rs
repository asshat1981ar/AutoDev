use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::ObjectiveRecord;

pub const PUBLIC_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicObjectiveSummary {
    pub schema_version: String,
    pub id: String,
    pub repository: String,
    pub description: String,
    pub branch: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicObjectiveCreate {
    pub repository: String,
    pub description: String,
    #[serde(default)]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicObjectiveEvent {
    pub schema_version: String,
    pub event_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: String,
    pub objective_id: String,
    pub run_id: Option<String>,
    pub task_id: Option<String>,
    pub data: Value,
}

impl PublicObjectiveEvent {
    pub fn queued(record: &ObjectiveRecord) -> Self {
        Self {
            schema_version: PUBLIC_SCHEMA_VERSION.to_string(),
            event_id: Uuid::new_v4().to_string(),
            event_type: "objective_queued".to_string(),
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            objective_id: record.id.clone(),
            run_id: None,
            task_id: None,
            data: json!({
                "repository": record.repository,
                "branch": record.branch,
                "status": record.status,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicProtocolError {
    pub schema_version: String,
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub correlation_id: Option<String>,
}
