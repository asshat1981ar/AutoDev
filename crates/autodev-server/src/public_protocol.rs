use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicProtocolError {
    pub schema_version: String,
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub correlation_id: Option<String>,
}
