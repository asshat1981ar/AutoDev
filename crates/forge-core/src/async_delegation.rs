use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

/// Explicit concurrency/security classification for delegated work.
///
/// v0 permits only read-only, compute, and external-observation tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationClass {
    ReadOnly,
    Compute,
    ExternalObservation,
    WorkspaceMutation,
    Irreversible,
}

impl DelegationClass {
    pub fn is_v0_eligible(self) -> bool {
        matches!(
            self,
            Self::ReadOnly | Self::Compute | Self::ExternalObservation
        )
    }

    fn wire_name(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Compute => "compute",
            Self::ExternalObservation => "external_observation",
            Self::WorkspaceMutation => "workspace_mutation",
            Self::Irreversible => "irreversible",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationRisk {
    Low,
    Medium,
    High,
}

impl DelegationRisk {
    fn wire_name(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Logical agent assignment produced by trusted dispatch planning.
///
/// This records the assignment that ForgeCore selected. It is not an
/// authorization grant and cannot mint additional capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationAssignment {
    agent_id: String,
    capabilities: Vec<String>,
}

impl DelegationAssignment {
    pub fn new(
        agent_id: impl Into<String>,
        capabilities: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, DelegationValidationError> {
        let agent_id = agent_id.into();
        require_non_blank(&agent_id, DelegationValidationError::MissingAgentId)?;

        Ok(Self {
            agent_id,
            capabilities: normalize_capabilities(capabilities)?,
        })
    }

    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    fn has_capability(&self, capability: &str) -> bool {
        self.capabilities
            .binary_search_by(|candidate| candidate.as_str().cmp(capability))
            .is_ok()
    }
}

/// Immutable, provider-neutral unit of delegated work.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DelegatedTask {
    task_id: String,
    correlation_id: String,
    parent_task_id: Option<String>,
    assignment: DelegationAssignment,
    class: DelegationClass,
    required_capabilities: Vec<String>,
    risk: DelegationRisk,
    provider: String,
    timeout_ms: u64,
    payload: Value,
    input_fingerprint: String,
}

impl DelegatedTask {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        task_id: impl Into<String>,
        correlation_id: impl Into<String>,
        parent_task_id: Option<String>,
        assignment: DelegationAssignment,
        class: DelegationClass,
        required_capabilities: impl IntoIterator<Item = impl Into<String>>,
        risk: DelegationRisk,
        provider: impl Into<String>,
        timeout_ms: u64,
        payload: Value,
    ) -> Result<Self, DelegationValidationError> {
        let task_id = task_id.into();
        let correlation_id = correlation_id.into();
        let provider = provider.into();

        require_non_blank(&task_id, DelegationValidationError::MissingTaskId)?;
        require_non_blank(
            &correlation_id,
            DelegationValidationError::MissingCorrelationId,
        )?;
        require_non_blank(&provider, DelegationValidationError::MissingProvider)?;

        if let Some(parent) = parent_task_id.as_deref() {
            require_non_blank(parent, DelegationValidationError::InvalidParentTaskId)?;
        }

        if timeout_ms == 0 {
            return Err(DelegationValidationError::ZeroTimeout);
        }

        let required_capabilities = normalize_capabilities(required_capabilities)?;
        for capability in &required_capabilities {
            if !assignment.has_capability(capability) {
                return Err(DelegationValidationError::CapabilityOutsideAssignment(
                    capability.clone(),
                ));
            }
        }

        let mut task = Self {
            task_id,
            correlation_id,
            parent_task_id,
            assignment,
            class,
            required_capabilities,
            risk,
            provider,
            timeout_ms,
            payload,
            input_fingerprint: String::new(),
        };
        task.input_fingerprint = task.compute_input_fingerprint();
        Ok(task)
    }

    /// Revalidates deserialized task state and applies the v0 fail-closed gate.
    pub fn validate_for_v0(&self) -> Result<(), DelegationValidationError> {
        if !self.class.is_v0_eligible() {
            return Err(DelegationValidationError::UnsupportedClass(self.class));
        }

        require_non_blank(&self.task_id, DelegationValidationError::MissingTaskId)?;
        require_non_blank(
            &self.correlation_id,
            DelegationValidationError::MissingCorrelationId,
        )?;
        require_non_blank(&self.provider, DelegationValidationError::MissingProvider)?;
        require_non_blank(
            &self.assignment.agent_id,
            DelegationValidationError::MissingAgentId,
        )?;

        if let Some(parent) = self.parent_task_id.as_deref() {
            require_non_blank(parent, DelegationValidationError::InvalidParentTaskId)?;
        }

        if self.timeout_ms == 0 {
            return Err(DelegationValidationError::ZeroTimeout);
        }

        for capability in &self.required_capabilities {
            require_non_blank(capability, DelegationValidationError::BlankCapability)?;
            if !self.assignment.has_capability(capability) {
                return Err(DelegationValidationError::CapabilityOutsideAssignment(
                    capability.clone(),
                ));
            }
        }

        if self.input_fingerprint != self.compute_input_fingerprint() {
            return Err(DelegationValidationError::InputFingerprintMismatch);
        }

        Ok(())
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn correlation_id(&self) -> &str {
        &self.correlation_id
    }

    pub fn parent_task_id(&self) -> Option<&str> {
        self.parent_task_id.as_deref()
    }

    pub fn assignment(&self) -> &DelegationAssignment {
        &self.assignment
    }

    pub fn class(&self) -> DelegationClass {
        self.class
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }

    pub fn risk(&self) -> DelegationRisk {
        self.risk
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }

    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    pub fn payload(&self) -> &Value {
        &self.payload
    }

    pub fn input_fingerprint(&self) -> &str {
        &self.input_fingerprint
    }

    fn compute_input_fingerprint(&self) -> String {
        let mut hasher = Sha256::new();

        hash_component(&mut hasher, "task_id", self.task_id.as_bytes());
        hash_component(
            &mut hasher,
            "correlation_id",
            self.correlation_id.as_bytes(),
        );
        hash_component(
            &mut hasher,
            "parent_task_id",
            self.parent_task_id
                .as_deref()
                .unwrap_or_default()
                .as_bytes(),
        );
        hash_component(
            &mut hasher,
            "assignment.agent_id",
            self.assignment.agent_id.as_bytes(),
        );
        for capability in &self.assignment.capabilities {
            hash_component(&mut hasher, "assignment.capability", capability.as_bytes());
        }
        hash_component(&mut hasher, "class", self.class.wire_name().as_bytes());
        for capability in &self.required_capabilities {
            hash_component(&mut hasher, "required_capability", capability.as_bytes());
        }
        hash_component(&mut hasher, "risk", self.risk.wire_name().as_bytes());
        hash_component(&mut hasher, "provider", self.provider.as_bytes());
        hash_component(
            &mut hasher,
            "timeout_ms",
            self.timeout_ms.to_string().as_bytes(),
        );
        hash_component(
            &mut hasher,
            "payload",
            canonical_json(&self.payload).as_bytes(),
        );

        to_lower_hex(&hasher.finalize())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DelegatedOutput {
    pub value: Value,
    pub evidence_fingerprint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationBlockReason {
    UnsupportedClass,
    InvalidTask,
    PolicyDenied,
    CapabilityDenied,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DelegatedOutcome {
    Completed {
        output: DelegatedOutput,
    },
    Blocked {
        reason: DelegationBlockReason,
        detail: String,
    },
    Failed {
        detail: String,
    },
    TimedOut {
        timeout_ms: u64,
    },
    Cancelled {
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DelegatedResult {
    pub task_id: String,
    pub correlation_id: String,
    pub provider: String,
    pub class: DelegationClass,
    pub outcome: DelegatedOutcome,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DelegationBatchResult {
    pub results: Vec<DelegatedResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DelegationValidationError {
    #[error("delegated task is missing task_id")]
    MissingTaskId,
    #[error("delegated task is missing correlation_id")]
    MissingCorrelationId,
    #[error("delegated task parent_task_id cannot be blank")]
    InvalidParentTaskId,
    #[error("delegated task is missing provider")]
    MissingProvider,
    #[error("delegated task assignment is missing agent_id")]
    MissingAgentId,
    #[error("delegated task capability cannot be blank")]
    BlankCapability,
    #[error("delegated task timeout must be greater than zero")]
    ZeroTimeout,
    #[error("delegation class is not eligible for asynchronous v0 execution: {0:?}")]
    UnsupportedClass(DelegationClass),
    #[error("required capability is outside the dispatch assignment: {0}")]
    CapabilityOutsideAssignment(String),
    #[error("duplicate delegated task id: {0}")]
    DuplicateTaskId(String),
    #[error("delegated task input fingerprint does not match task contents")]
    InputFingerprintMismatch,
}

pub fn validate_batch_for_v0(tasks: &[DelegatedTask]) -> Result<(), DelegationValidationError> {
    let mut ids = BTreeSet::new();

    for task in tasks {
        if !ids.insert(task.task_id().to_owned()) {
            return Err(DelegationValidationError::DuplicateTaskId(
                task.task_id().to_owned(),
            ));
        }
        task.validate_for_v0()?;
    }

    Ok(())
}

fn normalize_capabilities(
    capabilities: impl IntoIterator<Item = impl Into<String>>,
) -> Result<Vec<String>, DelegationValidationError> {
    let mut normalized = BTreeSet::new();

    for capability in capabilities {
        let capability = capability.into();
        require_non_blank(&capability, DelegationValidationError::BlankCapability)?;
        normalized.insert(capability);
    }

    Ok(normalized.into_iter().collect())
}

fn require_non_blank(
    value: &str,
    error: DelegationValidationError,
) -> Result<(), DelegationValidationError> {
    if value.trim().is_empty() {
        return Err(error);
    }
    Ok(())
}

fn hash_component(hasher: &mut Sha256, label: &str, value: &[u8]) {
    hasher.update((label.len() as u64).to_be_bytes());
    hasher.update(label.as_bytes());
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

fn to_lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => {
            serde_json::to_string(value).expect("serializing a JSON string to String cannot fail")
        }
        Value::Array(values) => {
            let inner = values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{inner}]")
        }
        Value::Object(values) => {
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();

            let inner = keys
                .into_iter()
                .map(|key| {
                    let encoded_key = serde_json::to_string(key)
                        .expect("serializing a JSON object key cannot fail");
                    let encoded_value = canonical_json(
                        values
                            .get(key)
                            .expect("key collected from object must still exist"),
                    );
                    format!("{encoded_key}:{encoded_value}")
                })
                .collect::<Vec<_>>()
                .join(",");

            format!("{{{inner}}}")
        }
    }
}
