//! Evidence-driven development execution loop.
//!
//! This module composes the existing execution envelope, policy boundary,
//! trusted executor, evidence store, and independent verification fabric. It
//! intentionally does not replace the durable task orchestrator; it provides
//! the execution primitive that the orchestrator can delegate to.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::envelope::{EnvelopeError, EnvelopeState, ExecutionEnvelope};
use crate::evidence::{
    record_from, EvidenceStore, ExecutionResult, ExecutionStatus, PolicyOutcome,
};
use crate::policy::{evaluate_policy, has_required_capability, PolicyDecision};
use crate::verification::{
    VerificationContext, VerificationFabric, VerificationReport, VerificationVerdict,
};
use crate::{execute, ExecutableAction, ExecutionError, Workspace};

/// The result of one bounded development-loop attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DevelopmentLoopOutcome {
    /// Independent verification passed and the envelope is terminally verified.
    Verified,
    /// Verification or execution failed, but another bounded attempt is allowed.
    Replanned,
    /// Verification or execution failed and the retry budget is exhausted.
    Exhausted,
}

/// Durable result returned by a development-loop attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevelopmentLoopResult {
    pub outcome: DevelopmentLoopOutcome,
    pub evidence_ref: String,
    pub verification: Option<VerificationReport>,
}

/// Errors that prevent an execution attempt from entering the trusted runtime.
#[derive(Debug, thiserror::Error)]
pub enum DevelopmentLoopError {
    #[error(transparent)]
    Envelope(#[from] EnvelopeError),
    #[error(transparent)]
    Execution(#[from] ExecutionError),
    #[error("action capability is not granted")]
    CapabilityDenied,
    #[error("envelope risk does not match action risk")]
    RiskMismatch,
    #[error("envelope capabilities do not include the action's required capability")]
    EnvelopeCapabilityDenied,
    #[error("envelope approval policy is weaker than the kernel policy")]
    ApprovalPolicyMismatch,
    #[error("approval reference is required by policy")]
    ApprovalRequired,
}

/// Trusted, evidence-driven ACT -> VERIFY -> REPLAN primitive.
pub struct DevelopmentLoop {
    pub verification: VerificationFabric,
    pub evidence: EvidenceStore,
}

impl DevelopmentLoop {
    pub fn new(verification: VerificationFabric) -> Self {
        Self {
            verification,
            evidence: EvidenceStore::new(),
        }
    }

    /// Execute one envelope attempt and independently verify the resulting
    /// workspace state.
    ///
    /// The model/agent's own success claim is never used as the completion
    /// signal. Only `VerificationFabric` can move the envelope to `Verified`.
    pub fn run_attempt(
        &mut self,
        envelope: &mut ExecutionEnvelope,
        workspace: &Workspace,
        verification_context: &VerificationContext,
    ) -> Result<DevelopmentLoopResult, DevelopmentLoopError> {
        envelope.validate()?;
        let policy_outcome = self.authorize(envelope)?;

        if envelope.lifecycle.state == EnvelopeState::Planned {
            envelope.transition(EnvelopeState::Authorized)?;
        }
        envelope.transition(EnvelopeState::Executing)?;

        let executed = execute(&ExecutableAction::new(
            envelope.action.clone(),
            workspace.clone(),
        ));

        let mut execution_result = match executed {
            Ok(result) => result,
            Err(error) => ExecutionResult {
                action_id: envelope.action.id.clone(),
                status: ExecutionStatus::Failed,
                started_at: Utc::now(),
                completed_at: Utc::now(),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                artifacts: vec![],
                verification: None,
                error: Some(error.to_string()),
            },
        };

        let verification = if execution_result.status == ExecutionStatus::Succeeded {
            envelope.transition(EnvelopeState::Verifying)?;
            let report = self.verification.run(verification_context);
            execution_result.verification =
                Some(serde_json::to_value(&report).expect("verification report serializes"));
            Some(report)
        } else {
            None
        };

        let record_id = format!(
            "evidence:{}:{}:{}",
            envelope.run_id, envelope.operation_id, envelope.lifecycle.attempt
        );
        let record = record_from(
            &record_id,
            &envelope.action,
            policy_outcome,
            &execution_result,
            vec![],
        );
        let evidence = self.evidence.insert(record);
        envelope.evidence.produced.push(evidence.record.id.clone());

        let independently_verified = verification
            .as_ref()
            .is_some_and(|report| report.overall == VerificationVerdict::Pass);

        if independently_verified {
            envelope.transition(EnvelopeState::Verified)?;
            return Ok(DevelopmentLoopResult {
                outcome: DevelopmentLoopOutcome::Verified,
                evidence_ref: evidence.record.id,
                verification,
            });
        }

        // Execution failure, failed verification, and skipped verification all
        // reject the attempt. Skipped verification is intentionally not success:
        // absence of evidence cannot satisfy acceptance criteria.
        envelope.transition(EnvelopeState::Rejected)?;
        envelope.transition(EnvelopeState::Replanning)?;

        if envelope.lifecycle.exhausted() {
            return Ok(DevelopmentLoopResult {
                outcome: DevelopmentLoopOutcome::Exhausted,
                evidence_ref: evidence.record.id,
                verification,
            });
        }

        envelope.transition(EnvelopeState::Planned)?;
        Ok(DevelopmentLoopResult {
            outcome: DevelopmentLoopOutcome::Replanned,
            evidence_ref: evidence.record.id,
            verification,
        })
    }

    fn authorize(&self, envelope: &ExecutionEnvelope) -> Result<PolicyOutcome, DevelopmentLoopError> {
        if envelope.policy.risk != envelope.action.risk {
            return Err(DevelopmentLoopError::RiskMismatch);
        }
        if !has_required_capability(&envelope.action) {
            return Err(DevelopmentLoopError::CapabilityDenied);
        }

        if let Some(required) = crate::Capability::for_action(envelope.action.action_type) {
            if !envelope
                .policy
                .capabilities
                .iter()
                .any(|cap| cap == &required)
            {
                return Err(DevelopmentLoopError::EnvelopeCapabilityDenied);
            }
        }

        match evaluate_policy(&envelope.action)? {
            PolicyDecision::Allow => {
                if envelope.policy.requires_approval {
                    if envelope.policy.is_authorized() {
                        Ok(PolicyOutcome::RequireApproval)
                    } else {
                        Err(DevelopmentLoopError::ApprovalRequired)
                    }
                } else {
                    Ok(PolicyOutcome::Allow)
                }
            }
            PolicyDecision::RequireApproval => {
                if !envelope.policy.requires_approval {
                    return Err(DevelopmentLoopError::ApprovalPolicyMismatch);
                }
                if envelope.policy.is_authorized() {
                    Ok(PolicyOutcome::RequireApproval)
                } else {
                    Err(DevelopmentLoopError::ApprovalRequired)
                }
            }
            PolicyDecision::Deny => Err(DevelopmentLoopError::CapabilityDenied),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::tempdir;

    use super::*;
    use crate::{
        mock_verifier, ActionType, AgentAction, Capability, ContextRefs, EvidenceBinding,
        Lifecycle, PolicyBinding, RiskLevel, VerificationKind,
    };

    fn envelope(task_id: &str, max_attempts: u32) -> ExecutionEnvelope {
        ExecutionEnvelope {
            operation_id: "op-1".into(),
            run_id: "run-1".into(),
            task_id: task_id.into(),
            action: AgentAction {
                id: "action-1".into(),
                task_id: task_id.into(),
                agent_id: "developer".into(),
                action_type: ActionType::WriteFile,
                reason: "write marker".into(),
                risk: RiskLevel::Low,
                capabilities: vec![Capability::WriteFile],
                payload: json!({"path":"marker.txt","content":"done"}),
                expected: json!({}),
            },
            context: ContextRefs::default(),
            policy: PolicyBinding {
                risk: RiskLevel::Low,
                capabilities: vec![Capability::WriteFile],
                requires_approval: false,
                approval_ref: None,
            },
            evidence: EvidenceBinding {
                required: vec!["unit_tests".into()],
                produced: vec![],
            },
            lifecycle: Lifecycle::new(max_attempts),
        }
    }

    fn verification_context(path: &std::path::Path) -> VerificationContext {
        VerificationContext {
            workspace: path.to_string_lossy().into_owned(),
            changed: vec!["marker.txt".into()],
        }
    }

    #[test]
    fn independent_verifier_controls_completion() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 1024 * 1024).unwrap();
        let fabric = VerificationFabric::new().with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, true),
        );
        let mut loop_ = DevelopmentLoop::new(fabric);
        let mut env = envelope("task-1", 2);

        let result = loop_
            .run_attempt(&mut env, &workspace, &verification_context(dir.path()))
            .unwrap();

        assert_eq!(result.outcome, DevelopmentLoopOutcome::Verified);
        assert_eq!(env.lifecycle.state, EnvelopeState::Verified);
        assert_eq!(env.evidence.produced.len(), 1);
        assert!(loop_.evidence.get(&result.evidence_ref).unwrap().verify());
    }

    #[test]
    fn verifier_rejection_replans_even_when_execution_succeeds() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 1024 * 1024).unwrap();
        let fabric = VerificationFabric::new().with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, false),
        );
        let mut loop_ = DevelopmentLoop::new(fabric);
        let mut env = envelope("task-1", 2);

        let result = loop_
            .run_attempt(&mut env, &workspace, &verification_context(dir.path()))
            .unwrap();

        assert_eq!(result.outcome, DevelopmentLoopOutcome::Replanned);
        assert_eq!(env.lifecycle.state, EnvelopeState::Planned);
        assert_eq!(env.lifecycle.attempt, 2);
        assert_eq!(env.evidence.produced.len(), 1);
    }

    #[test]
    fn retry_budget_exhaustion_is_terminal_for_the_attempt_loop() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 1024 * 1024).unwrap();
        let fabric = VerificationFabric::new().with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, false),
        );
        let mut loop_ = DevelopmentLoop::new(fabric);
        let mut env = envelope("task-1", 1);

        let result = loop_
            .run_attempt(&mut env, &workspace, &verification_context(dir.path()))
            .unwrap();

        assert_eq!(result.outcome, DevelopmentLoopOutcome::Exhausted);
        assert_eq!(env.lifecycle.state, EnvelopeState::Replanning);
        assert_eq!(env.evidence.produced.len(), 1);
    }

    #[test]
    fn envelope_policy_cannot_escalate_or_drop_required_capability() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 1024 * 1024).unwrap();
        let fabric = VerificationFabric::new().with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, true),
        );
        let mut loop_ = DevelopmentLoop::new(fabric);
        let mut env = envelope("task-1", 2);
        env.policy.risk = RiskLevel::High;

        let err = loop_
            .run_attempt(&mut env, &workspace, &verification_context(dir.path()))
            .unwrap_err();
        assert!(matches!(err, DevelopmentLoopError::RiskMismatch));

        env.policy.risk = RiskLevel::Low;
        env.policy.capabilities.clear();
        let err = loop_
            .run_attempt(&mut env, &workspace, &verification_context(dir.path()))
            .unwrap_err();
        assert!(matches!(
            err,
            DevelopmentLoopError::EnvelopeCapabilityDenied
        ));
    }

    #[test]
    fn kernel_approval_requirement_cannot_be_disabled_by_envelope() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 1024 * 1024).unwrap();
        let fabric = VerificationFabric::new().with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, true),
        );
        let mut loop_ = DevelopmentLoop::new(fabric);
        let mut env = envelope("task-1", 2);
        env.action.risk = RiskLevel::High;
        env.policy.risk = RiskLevel::High;
        env.policy.requires_approval = false;

        let err = loop_
            .run_attempt(&mut env, &workspace, &verification_context(dir.path()))
            .unwrap_err();
        assert!(matches!(
            err,
            DevelopmentLoopError::ApprovalPolicyMismatch
        ));
    }

    #[test]
    fn approved_high_risk_action_records_approval_policy() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 1024 * 1024).unwrap();
        let fabric = VerificationFabric::new().with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, true),
        );
        let mut loop_ = DevelopmentLoop::new(fabric);
        let mut env = envelope("task-1", 2);
        env.action.risk = RiskLevel::High;
        env.policy.risk = RiskLevel::High;
        env.policy.requires_approval = true;
        env.policy.approval_ref = Some("approval-1".into());

        let result = loop_
            .run_attempt(&mut env, &workspace, &verification_context(dir.path()))
            .unwrap();

        let evidence = loop_.evidence.get(&result.evidence_ref).unwrap();
        assert_eq!(evidence.record.policy, PolicyOutcome::RequireApproval);
    }

    #[test]
    fn missing_verifier_is_not_treated_as_success() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 1024 * 1024).unwrap();
        let mut loop_ = DevelopmentLoop::new(VerificationFabric::new());
        let mut env = envelope("task-1", 2);

        let result = loop_
            .run_attempt(&mut env, &workspace, &verification_context(dir.path()))
            .unwrap();

        assert_eq!(result.outcome, DevelopmentLoopOutcome::Replanned);
        assert_eq!(
            result.verification.unwrap().overall,
            VerificationVerdict::Skipped
        );
    }
}
