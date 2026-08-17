//! Kernel-owned execution authority.
//!
//! This module deliberately has no serialization support and no dependency on
//! `AgentAction`. Values are created only by trusted orchestration/policy code
//! and consumed by execution adapters. Untrusted protocol capability strings
//! must be explicitly translated into this closed set by trusted code.

/// A capability the trusted kernel may grant for execution.
///
/// Unlike the protocol-level `Capability`, this enum contains only privileges
/// that can authorize effects. Approval is represented separately so it cannot
/// be confused with an execution capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrantedCapability {
    ReadFile,
    WriteFile,
    PatchFile,
    Execute,
    Git,
    GitWrite,
    GitDestructive,
    Mcp,
    RunTest,
    RequestApproval,
}

impl GrantedCapability {
    /// Stable wire name used when recording trusted authority evidence.
    pub fn as_str(self) -> &'static str {
        match self {
            GrantedCapability::ReadFile => "read_file",
            GrantedCapability::WriteFile => "write_file",
            GrantedCapability::PatchFile => "patch_file",
            GrantedCapability::Execute => "execute",
            GrantedCapability::Git => "git",
            GrantedCapability::GitWrite => "git:write",
            GrantedCapability::GitDestructive => "git:destructive",
            GrantedCapability::Mcp => "mcp",
            GrantedCapability::RunTest => "run_test",
            GrantedCapability::RequestApproval => "request_approval",
        }
    }
}

/// Trusted authorization material bound to one execution request.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionAuthority {
    granted_capabilities: Vec<GrantedCapability>,
    approval_ref: Option<String>,
}

impl ExecutionAuthority {
    /// Construct a fail-closed authority with no capabilities or approval.
    pub fn none() -> Self {
        Self::default()
    }

    /// Construct trusted capability authority without human approval.
    pub fn granted(capabilities: Vec<GrantedCapability>) -> Self {
        Self {
            granted_capabilities: deduplicate(capabilities),
            approval_ref: None,
        }
    }

    /// Construct trusted capability authority with an approval reference.
    pub fn with_approval(
        capabilities: Vec<GrantedCapability>,
        approval_ref: impl Into<String>,
    ) -> Self {
        Self {
            granted_capabilities: deduplicate(capabilities),
            approval_ref: Some(approval_ref.into()),
        }
    }

    /// Return whether this authority explicitly grants `capability`.
    pub fn allows(&self, capability: GrantedCapability) -> bool {
        self.granted_capabilities.contains(&capability)
    }

    /// Return the immutable set of capabilities granted by trusted code.
    pub fn granted_capabilities(&self) -> &[GrantedCapability] {
        &self.granted_capabilities
    }

    /// Return the approval reference when non-blank.
    pub fn approval_ref(&self) -> Option<&str> {
        self.approval_ref
            .as_deref()
            .filter(|reference| !reference.trim().is_empty())
    }

    /// Human approval exists only when a non-blank trusted reference is bound.
    pub fn is_approved(&self) -> bool {
        self.approval_ref().is_some()
    }
}

fn deduplicate(capabilities: Vec<GrantedCapability>) -> Vec<GrantedCapability> {
    let mut unique = Vec::with_capacity(capabilities.len());
    for capability in capabilities {
        if !unique.contains(&capability) {
            unique.push(capability);
        }
    }
    unique
}
