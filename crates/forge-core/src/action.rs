//! Typed protocol types for the AutoDev agent-action protocol.
//!
//! These types serialize to the JSON contracts in `protocols/*.schema.json`.
//! Field names and enum values follow the schemas exactly so that any
//! language-neutral consumer can interoperate.

use serde::{Deserialize, Serialize};

/// The kinds of operations an agent may request.
///
/// Serde uses `snake_case` so the serialized value matches the schema enum:
/// `read_file`, `write_file`, `patch_file`, `execute`, `git`, `mcp`,
/// `run_test`, `request_approval`. Unknown variants are rejected by serde.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    ReadFile,
    WriteFile,
    PatchFile,
    Execute,
    Git,
    Mcp,
    RunTest,
    RequestApproval,
}

impl ActionType {
    /// The serialized wire name of this action type.
    pub fn as_str(self) -> &'static str {
        match self {
            ActionType::ReadFile => "read_file",
            ActionType::WriteFile => "write_file",
            ActionType::PatchFile => "patch_file",
            ActionType::Execute => "execute",
            ActionType::Git => "git",
            ActionType::Mcp => "mcp",
            ActionType::RunTest => "run_test",
            ActionType::RequestApproval => "request_approval",
        }
    }
}

/// Risk classification of an action.
///
/// Matches the schema enum: `low`, `medium`, `high`, `critical`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    /// The serialized wire name of this risk level.
    pub fn as_str(self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

/// A capability that may be granted to an agent.
///
/// Capabilities are declared, never inferred. Known variants map to the
/// platform's action types plus the special `approval:critical` capability.
/// Unknown capability strings are preserved via [`Capability::Unknown`] so the
/// protocol remains forward-compatible while still being strongly typed for
/// the capabilities the kernel understands.
///
/// Serialization is a plain string (e.g. `"read_file"`), matching the schema's
/// `capabilities: string[]` contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    ReadFile,
    WriteFile,
    PatchFile,
    Execute,
    Git,
    /// Mutating Git operations (checkpoint, commit preparation / staging).
    GitWrite,
    /// Destructive Git operations (reset --hard, checkout, revert).
    GitDestructive,
    Mcp,
    RunTest,
    RequestApproval,
    ApprovalCritical,
    Unknown(String),
}

impl Capability {
    /// Parse a capability from its wire string.
    pub fn parse(value: &str) -> Self {
        match value {
            "read_file" => Capability::ReadFile,
            "write_file" => Capability::WriteFile,
            "patch_file" => Capability::PatchFile,
            "execute" => Capability::Execute,
            "git" => Capability::Git,
            "git:write" => Capability::GitWrite,
            "git:destructive" => Capability::GitDestructive,
            "mcp" => Capability::Mcp,
            "run_test" => Capability::RunTest,
            "request_approval" => Capability::RequestApproval,
            "approval:critical" => Capability::ApprovalCritical,
            other => Capability::Unknown(other.to_string()),
        }
    }

    /// The wire string of this capability.
    pub fn as_str(&self) -> &str {
        match self {
            Capability::ReadFile => "read_file",
            Capability::WriteFile => "write_file",
            Capability::PatchFile => "patch_file",
            Capability::Execute => "execute",
            Capability::Git => "git",
            Capability::GitWrite => "git:write",
            Capability::GitDestructive => "git:destructive",
            Capability::Mcp => "mcp",
            Capability::RunTest => "run_test",
            Capability::RequestApproval => "request_approval",
            Capability::ApprovalCritical => "approval:critical",
            Capability::Unknown(value) => value,
        }
    }

    /// The capability required to perform an [`ActionType`], if such a mapping
    /// exists.
    pub fn for_action(action: ActionType) -> Option<Capability> {
        match action {
            ActionType::ReadFile => Some(Capability::ReadFile),
            ActionType::WriteFile => Some(Capability::WriteFile),
            ActionType::PatchFile => Some(Capability::PatchFile),
            ActionType::Execute => Some(Capability::Execute),
            ActionType::Git => Some(Capability::Git),
            ActionType::Mcp => Some(Capability::Mcp),
            ActionType::RunTest => Some(Capability::RunTest),
            ActionType::RequestApproval => Some(Capability::RequestApproval),
        }
    }
}

impl serde::Serialize for Capability {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Capability {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Capability::parse(&s))
    }
}

/// A typed, validated agent action.
///
/// Field names match `agent-action.schema.json`. `action_type` serializes to
/// the `type` field. `additionalProperties: false` in the schema is enforced
/// with `#[serde(deny_unknown_fields)]` so unknown fields are rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentAction {
    pub id: String,
    pub task_id: String,
    pub agent_id: String,
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub reason: String,
    pub risk: RiskLevel,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub expected: serde_json::Value,
}
