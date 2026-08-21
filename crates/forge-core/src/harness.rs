//! Declarative harness profiles for AutoDev development workflows.
//!
//! Harness configuration expresses intent, orchestration stages, asset
//! references, and verification contracts. It is deliberately authority-free:
//! profiles cannot authorize or execute effects and cannot mark their own work
//! verified. ForgeCore policy and execution remain the trusted boundary.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Broad family of a development harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessKind {
    Sdlc,
    Agile,
    Innovation,
    Optimizer,
    Meta,
}

/// Normalized kind for an asset referenced by a harness profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessAssetKind {
    Skill,
    AgentProfile,
    Tool,
    McpServer,
    Hook,
    Prompt,
    Policy,
    Workflow,
    Evaluator,
    ContextProvider,
}

/// Versioned reference to a declarative harness asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessAssetRef {
    pub id: String,
    pub version: String,
    pub kind: HarnessAssetKind,
    pub required: bool,
}

/// One ordered stage in a harness profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessStage {
    pub id: String,
    pub objective: String,
    #[serde(default)]
    pub assets: Vec<HarnessAssetRef>,
    #[serde(default)]
    pub verification: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_group: Option<String>,
    #[serde(default)]
    pub approval_gate: bool,
}

/// A versioned, authority-free development harness profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessProfile {
    pub id: String,
    pub version: String,
    pub name: String,
    pub kind: HarnessKind,
    pub objective: String,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub stages: Vec<HarnessStage>,
    #[serde(default)]
    pub success_metrics: Vec<String>,
    #[serde(default)]
    pub memory_policy: Vec<String>,
    #[serde(default)]
    pub improvement_policy: Vec<String>,
}

/// Validation and registration errors for harness configuration.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HarnessError {
    #[error("harness profile '{profile_id}' has empty required field '{field}'")]
    EmptyField {
        profile_id: String,
        field: &'static str,
    },
    #[error("harness profile '{profile_id}' contains duplicate stage '{stage_id}'")]
    DuplicateStage {
        profile_id: String,
        stage_id: String,
    },
    #[error("harness profile '{profile_id}' stage '{stage_id}' has no verification contract")]
    MissingVerification {
        profile_id: String,
        stage_id: String,
    },
    #[error("harness profile '{profile_id}' stage '{stage_id}' has invalid asset: {reason}")]
    InvalidAsset {
        profile_id: String,
        stage_id: String,
        reason: String,
    },
    #[error("harness profile '{0}' is already registered")]
    DuplicateProfile(String),
}

impl HarnessProfile {
    /// Validate structural invariants without executing or authorizing effects.
    pub fn validate(&self) -> Result<(), HarnessError> {
        self.require_non_empty("id", &self.id)?;
        self.require_non_empty("version", &self.version)?;
        self.require_non_empty("name", &self.name)?;
        self.require_non_empty("objective", &self.objective)?;
        if self.triggers.is_empty() || self.triggers.iter().any(|term| term.trim().is_empty()) {
            return Err(HarnessError::EmptyField {
                profile_id: self.id.clone(),
                field: "triggers",
            });
        }
        if self.stages.is_empty() {
            return Err(HarnessError::EmptyField {
                profile_id: self.id.clone(),
                field: "stages",
            });
        }

        let mut stage_ids = BTreeSet::new();
        for stage in &self.stages {
            if stage.id.trim().is_empty() {
                return Err(HarnessError::EmptyField {
                    profile_id: self.id.clone(),
                    field: "stage.id",
                });
            }
            if !stage_ids.insert(stage.id.as_str()) {
                return Err(HarnessError::DuplicateStage {
                    profile_id: self.id.clone(),
                    stage_id: stage.id.clone(),
                });
            }
            if stage.objective.trim().is_empty() {
                return Err(HarnessError::EmptyField {
                    profile_id: self.id.clone(),
                    field: "stage.objective",
                });
            }
            if stage.verification.is_empty()
                || stage
                    .verification
                    .iter()
                    .any(|contract| contract.trim().is_empty())
            {
                return Err(HarnessError::MissingVerification {
                    profile_id: self.id.clone(),
                    stage_id: stage.id.clone(),
                });
            }
            for asset in &stage.assets {
                if asset.id.trim().is_empty() {
                    return Err(HarnessError::InvalidAsset {
                        profile_id: self.id.clone(),
                        stage_id: stage.id.clone(),
                        reason: "asset id must not be empty".to_string(),
                    });
                }
                if asset.version.trim().is_empty() {
                    return Err(HarnessError::InvalidAsset {
                        profile_id: self.id.clone(),
                        stage_id: stage.id.clone(),
                        reason: "asset version must not be empty".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn require_non_empty(&self, field: &'static str, value: &str) -> Result<(), HarnessError> {
        if value.trim().is_empty() {
            Err(HarnessError::EmptyField {
                profile_id: self.id.clone(),
                field,
            })
        } else {
            Ok(())
        }
    }
}

/// Deterministic registry of validated harness profiles.
#[derive(Debug, Clone, Default)]
pub struct HarnessRegistry {
    profiles: Vec<HarnessProfile>,
}

impl HarnessRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, profile: HarnessProfile) -> Result<(), HarnessError> {
        profile.validate()?;
        if self
            .profiles
            .iter()
            .any(|candidate| candidate.id == profile.id)
        {
            return Err(HarnessError::DuplicateProfile(profile.id));
        }
        self.profiles.push(profile);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&HarnessProfile> {
        self.profiles.iter().find(|profile| profile.id == id)
    }

    pub fn profiles(&self) -> &[HarnessProfile] {
        &self.profiles
    }
}
