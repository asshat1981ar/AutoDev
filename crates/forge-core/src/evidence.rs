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

/// The hash algorithm used to fingerprint an artifact or record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactHashAlgo {
    Sha256,
}

/// A content hash of an artifact or record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactHash {
    pub algo: ArtifactHashAlgo,
    /// Lowercase hex digest.
    pub digest: String,
}

impl ArtifactHash {
    /// Compute a SHA-256 hash over `bytes`.
    pub fn sha256(bytes: &[u8]) -> Self {
        ArtifactHash {
            algo: ArtifactHashAlgo::Sha256,
            digest: sha256_hex(bytes),
        }
    }
}

/// An artifact produced by an execution (a file, blob, or verification output).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    /// Stable unique identifier for the artifact.
    pub id: String,
    /// Human-readable name (e.g. a path or label).
    pub name: String,
    /// Coarse kind of the artifact (e.g. "file", "diff", "stdout").
    pub kind: String,
    /// Content hash for provenance.
    pub hash: ArtifactHash,
    /// Size in bytes.
    pub size: u64,
    /// Where the artifact lives (workspace-relative path or `null` for inline).
    pub path: Option<String>,
    /// When the artifact was produced.
    pub created_at: DateTime<Utc>,
}

/// The policy decision that authorized (or refused) an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyOutcome {
    Allow,
    RequireApproval,
    Deny,
}

/// The complete, traceable record of one executed action.
///
/// This is the unit of provenance: it captures the full chain
/// `Task → Agent → Action → Policy Decision → Execution → Artifact → Verification`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Provenance identifier for this record (stable, unique).
    pub id: String,
    /// The task this execution belongs to.
    pub task_id: String,
    /// The agent that emitted the action.
    pub agent_id: String,
    /// The action id (from the originating `AgentAction`).
    pub action_id: String,
    /// A serialized snapshot of the action that was executed.
    pub action: serde_json::Value,
    /// The policy decision that governed this execution.
    pub policy: PolicyOutcome,
    /// The final execution status.
    pub status: ExecutionStatus,
    /// When execution started.
    pub started_at: DateTime<Utc>,
    /// When execution completed.
    pub completed_at: DateTime<Utc>,
    /// Error information, if the execution failed.
    pub error: Option<ExecutionErrorInfo>,
    /// Artifacts produced by the execution.
    pub artifacts: Vec<Artifact>,
    /// Verification payload (hashes, diffs, etc.).
    pub verification: Option<serde_json::Value>,
}

/// Serialized error information captured on a failed execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionErrorInfo {
    /// A stable error kind string.
    pub kind: String,
    /// A human-readable message.
    pub message: String,
}

/// An immutable, self-describing evidence package.
///
/// Holds an [`ExecutionRecord`] plus a content hash over its canonical JSON, so
/// the record can be verified for tampering and reconstructed independently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    /// The underlying record.
    pub record: ExecutionRecord,
    /// SHA-256 over the canonical JSON of `record`.
    pub fingerprint: ArtifactHash,
}

impl Evidence {
    /// Compute the fingerprint over a record and wrap it.
    pub fn from_record(record: ExecutionRecord) -> Self {
        let canonical = serde_json::to_vec(&record).expect("record serializes");
        let fingerprint = ArtifactHash::sha256(&canonical);
        Evidence {
            record,
            fingerprint,
        }
    }

    /// Recompute the fingerprint and check it matches the recorded one.
    pub fn verify(&self) -> bool {
        let canonical = serde_json::to_vec(&self.record).expect("record serializes");
        ArtifactHash::sha256(&canonical) == self.fingerprint
    }
}
/// An append-only store of execution evidence.
///
/// This is the persistence boundary for this stage. It keeps records in memory
/// (deterministic and testable) and each [`Evidence`] serializes to standalone
/// JSON, so persisting to files later is a one-line change and swapping in
/// SQLite behind this interface is straightforward. A graph database is
/// deliberately not used.
#[derive(Debug, Clone, Default)]
pub struct EvidenceStore {
    records: Vec<Evidence>,
}

impl EvidenceStore {
    /// Create an empty store.
    pub fn new() -> Self {
        EvidenceStore::default()
    }

    /// Insert a record, wrapping it in [`Evidence`] with a content fingerprint.
    /// Returns the evidence (and its id) for chaining.
    pub fn insert(&mut self, record: ExecutionRecord) -> Evidence {
        let evidence = Evidence::from_record(record);
        self.records.push(evidence.clone());
        evidence
    }

    /// All records, oldest first.
    pub fn records(&self) -> &[Evidence] {
        &self.records
    }

    /// Look up an [`Evidence`] by its record id.
    pub fn get(&self, id: &str) -> Option<&Evidence> {
        self.records.iter().find(|e| e.record.id == id)
    }

    /// Look up evidence by the originating action id.
    pub fn by_action_id(&self, action_id: &str) -> Option<&Evidence> {
        self.records
            .iter()
            .find(|e| e.record.action_id == action_id)
    }

    /// Reconstruct the execution chain for `action_id`, oldest first.
    ///
    /// Returns the records for the given action (usually one) joined by task and
    /// agent, in chronological order.
    pub fn chain_for_action(&self, action_id: &str) -> Vec<&Evidence> {
        let mut chain: Vec<&Evidence> = self
            .records
            .iter()
            .filter(|e| e.record.action_id == action_id)
            .collect();
        chain.sort_by_key(|a| a.record.started_at);
        chain
    }

    /// Number of records stored.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Build an [`ExecutionRecord`] from an action, its policy outcome, a result,
/// and optional artifacts.
///
/// This is the single place that turns an executed action + result into a
/// traceable record, so identifiers and timestamps are captured consistently.
pub fn record_from(
    id: &str,
    action: &crate::action::AgentAction,
    policy: PolicyOutcome,
    result: &ExecutionResult,
    artifacts: Vec<Artifact>,
) -> ExecutionRecord {
    ExecutionRecord {
        id: id.to_string(),
        task_id: action.task_id.clone(),
        agent_id: action.agent_id.clone(),
        action_id: action.id.clone(),
        action: serde_json::to_value(action).expect("action serializes"),
        policy,
        status: result.status,
        started_at: result.started_at,
        completed_at: result.completed_at,
        error: result.error.as_ref().map(|e| ExecutionErrorInfo {
            kind: "execution_error".to_string(),
            message: e.clone(),
        }),
        artifacts,
        verification: result.verification.clone(),
    }
}

/// Reconstruct the originating action id from a record's action snapshot.
///
/// Returns the action's `id` field, or `None` if the snapshot is malformed.
pub fn action_id_from_record(record: &ExecutionRecord) -> Option<String> {
    record
        .action
        .get("id")
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Reconstruct the originating action type from a record's action snapshot.
pub fn action_type_from_record(record: &ExecutionRecord) -> Option<String> {
    record
        .action
        .get("type")
        .and_then(|v| v.as_str())
        .map(String::from)
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

    #[test]
    fn artifact_hash_hashes_content() {
        let h = ArtifactHash::sha256(b"hello");
        assert_eq!(h.algo, ArtifactHashAlgo::Sha256);
        assert_eq!(h.digest, sha256_hex(b"hello"));
    }

    #[test]
    fn evidence_fingerprint_verifies_and_detects_tampering() {
        let record = record_from(
            "rec-1",
            &crate::action::AgentAction {
                id: "a1".to_string(),
                task_id: "t1".to_string(),
                agent_id: "g1".to_string(),
                action_type: crate::action::ActionType::ReadFile,
                reason: "read".to_string(),
                risk: crate::action::RiskLevel::Low,
                capabilities: vec![],
                payload: serde_json::json!({ "path": "a.txt" }),
                expected: serde_json::json!({}),
            },
            PolicyOutcome::Allow,
            &ExecutionResult {
                action_id: "a1".to_string(),
                status: ExecutionStatus::Succeeded,
                started_at: Utc::now(),
                completed_at: Utc::now(),
                exit_code: None,
                stdout: "hello".to_string(),
                stderr: String::new(),
                artifacts: vec![],
                verification: None,
                error: None,
            },
            vec![],
        );
        let evidence = Evidence::from_record(record);
        assert!(evidence.verify());

        // Tamper with the record -> fingerprint no longer matches.
        let mut tampered = evidence.clone();
        tampered.record.status = ExecutionStatus::Failed;
        assert_ne!(tampered.record, evidence.record);
        assert!(!tampered.verify());
    }

    #[test]
    fn store_inserts_and_looks_up() {
        let mut store = EvidenceStore::new();
        let record = record_from(
            "rec-1",
            &crate::action::AgentAction {
                id: "a1".to_string(),
                task_id: "t1".to_string(),
                agent_id: "g1".to_string(),
                action_type: crate::action::ActionType::WriteFile,
                reason: "write".to_string(),
                risk: crate::action::RiskLevel::Low,
                capabilities: vec![crate::action::Capability::WriteFile],
                payload: serde_json::json!({ "path": "a.txt", "content": "x" }),
                expected: serde_json::json!({}),
            },
            PolicyOutcome::Allow,
            &ExecutionResult {
                action_id: "a1".to_string(),
                status: ExecutionStatus::Succeeded,
                started_at: Utc::now(),
                completed_at: Utc::now(),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                artifacts: vec![],
                verification: None,
                error: None,
            },
            vec![],
        );
        store.insert(record);
        assert_eq!(store.len(), 1);
        assert!(store.get("rec-1").is_some());
        assert!(store.by_action_id("a1").is_some());
    }

    #[test]
    fn action_can_be_reconstructed_from_evidence() {
        let mut store = EvidenceStore::new();
        let action = crate::action::AgentAction {
            id: "a7".to_string(),
            task_id: "t7".to_string(),
            agent_id: "g7".to_string(),
            action_type: crate::action::ActionType::ReadFile,
            reason: "inspect".to_string(),
            risk: crate::action::RiskLevel::Low,
            capabilities: vec![crate::action::Capability::ReadFile],
            payload: serde_json::json!({ "path": "src/lib.rs" }),
            expected: serde_json::json!({}),
        };
        let record = record_from(
            "rec-7",
            &action,
            PolicyOutcome::Allow,
            &ExecutionResult {
                action_id: "a7".to_string(),
                status: ExecutionStatus::Succeeded,
                started_at: Utc::now(),
                completed_at: Utc::now(),
                exit_code: None,
                stdout: "content".to_string(),
                stderr: String::new(),
                artifacts: vec![],
                verification: None,
                error: None,
            },
            vec![],
        );
        store.insert(record);

        // Reconstruct the action from evidence: look up by action id, load the
        // snapshot, and deserialize back to an AgentAction.
        let evidence = store.by_action_id("a7").unwrap();
        let reconstructed: crate::action::AgentAction =
            serde_json::from_value(evidence.record.action.clone()).unwrap();
        assert_eq!(reconstructed, action);
        assert_eq!(action_id_from_record(&evidence.record).unwrap(), "a7");
        assert_eq!(
            action_type_from_record(&evidence.record).unwrap(),
            "read_file"
        );
    }
}
