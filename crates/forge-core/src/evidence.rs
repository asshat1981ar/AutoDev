//! Execution evidence.
//!
//! An [`ExecutionResult`] is the durable, schema-conformant evidence produced
//! by an execution adapter. It aligns with `execution-result.schema.json`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The status of an execution.
///
/// Matches the schema enum exactly: `accepted`, `denied`, `running`,
/// `succeeded`, `failed`, `cancelled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Accepted,
    Denied,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// Evidence returned by the execution layer.
///
/// Conforms to `execution-result.schema.json`. Fields not applicable to a
/// non-process operation (e.g. `exit_code`) serialize as `null`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionResult {
    pub action_id: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub artifacts: Vec<String>,
    pub verification: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Metadata captured about a read file, embedded in the result's verification
/// payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadMetadata {
    /// The canonical path that was read.
    pub path: String,
    /// SHA-256 hex digest of the file contents.
    pub sha256: String,
    /// Size of the file in bytes.
    pub size: u64,
    /// Last-modified timestamp, if available.
    pub modified_at: Option<DateTime<Utc>>,
}

/// Compute the lowercase hex SHA-256 digest of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_matches_known_vector() {
        // sha256("abc") = ba7816bf8f01cfea414140de5dae2223...
        let digest = sha256_hex(b"abc");
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn result_round_trips_to_json() {
        let result = ExecutionResult {
            action_id: "a1".to_string(),
            status: ExecutionStatus::Succeeded,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            exit_code: None,
            stdout: "hello".to_string(),
            stderr: String::new(),
            artifacts: vec!["/ws/a.txt".to_string()],
            verification: Some(serde_json::json!({ "sha256": "abc" })),
            error: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["status"], "succeeded");
        assert_eq!(json["action_id"], "a1");
    }
}
