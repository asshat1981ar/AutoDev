//! Durable orchestration adapter for evidence-driven execution.
//!
//! This composes the existing `TaskGraph` scheduler with `DevelopmentLoop`.
//! The legacy orchestrator remains intact while this adapter proves the new
//! execution-envelope path end to end.

use std::collections::BTreeMap;

use chrono::Utc;

use crate::development_loop::{
    DevelopmentLoop, DevelopmentLoopError, DevelopmentLoopOutcome, DevelopmentLoopResult,
};
use crate::envelope::ExecutionEnvelope;
use crate::orchestrator::{
    Assigner, Checkpointer, Decomposer, Phase, TaskGraph, TaskNode, TaskStatus,
};
use crate::verification::VerificationContext;
use crate::Workspace;

/// Builds the initial envelope for a ready task.
pub type EnvelopeFactory = Box<dyn Fn(&TaskNode) -> ExecutionEnvelope>;

/// Serializable state owned by the verified orchestration path.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct VerifiedOrchestratorState {
    /// One durable envelope per task. Attempts and evidence references persist
    /// here across replanning cycles.
    pub envelopes: BTreeMap<String, ExecutionEnvelope>,
}

/// Errors raised while advancing the verifier-driven task loop.
#[derive(Debug, thiserror::Error)]
pub enum VerifiedOrchestratorError {
    #[error("unknown task '{0}'")]
    UnknownTask(String),
    #[error("task '{0}' is missing an agent assignment")]
    AssignmentFailed(String),
    #[error(transparent)]
    DevelopmentLoop(#[from] DevelopmentLoopError),
}

/// Durable task scheduler backed by `DevelopmentLoop`.
pub struct VerifiedOrchestrator {
    pub decomposer: Decomposer,
    pub assigner: Assigner,
    pub checkpointer: Checkpointer,
    pub development: DevelopmentLoop,
    pub envelope_factory: EnvelopeFactory,
    pub state: VerifiedOrchestratorState,
}

impl VerifiedOrchestrator {
    pub fn new(
        decomposer: Decomposer,
        assigner: Assigner,
        development: DevelopmentLoop,
        envelope_factory: EnvelopeFactory,
    ) -> Self {
        Self {
            decomposer,
            assigner,
            checkpointer: Checkpointer,
            development,
            envelope_factory,
            state: VerifiedOrchestratorState::default(),
        }
    }

    /// Advance one durable task attempt.
    ///
    /// The task graph remains the scheduler of record; the envelope map is the
    /// execution-attempt record. Verification, not execution success, controls
    /// completion.
    pub fn advance(
        &mut self,
        graph: &mut TaskGraph,
        workspace: &Workspace,
        verification_context: &VerificationContext,
    ) -> Result<Phase, VerifiedOrchestratorError> {
        self.checkpointer.checkpoint(graph);

        let task_id = match graph.next_ready().map(|task| task.id.clone()) {
            Some(id) => id,
            None if graph.root().status == TaskStatus::Planning => {
                self.decomposer.decompose(graph, &graph.root.clone());
                return Ok(Phase::Decompose);
            }
            None => return Ok(Phase::Checkpoint),
        };

        if self.assigner.assign(graph, &task_id).is_none() {
            return Err(VerifiedOrchestratorError::AssignmentFailed(task_id));
        }

        let task = graph
            .get(&task_id)
            .cloned()
            .ok_or_else(|| VerifiedOrchestratorError::UnknownTask(task_id.clone()))?;

        let envelope = self
            .state
            .envelopes
            .entry(task_id.clone())
            .or_insert_with(|| (self.envelope_factory)(&task));

        let result = match self
            .development
            .run_attempt(envelope, workspace, verification_context)
        {
            Ok(result) => result,
            Err(DevelopmentLoopError::ApprovalRequired) => {
                self.transition_task(
                    graph,
                    &task_id,
                    TaskStatus::Blocked,
                    "AUTHORIZE",
                    "waiting for approval",
                );
                return Ok(Phase::Act);
            }
            Err(error) => {
                self.transition_task(
                    graph,
                    &task_id,
                    TaskStatus::Failed,
                    "ACT",
                    &format!("execution boundary rejected task: {error}"),
                );
                return Err(error.into());
            }
        };

        self.apply_result(graph, &task_id, &result);
        Ok(match result.outcome {
            DevelopmentLoopOutcome::Verified => Phase::Verify,
            DevelopmentLoopOutcome::Replanned | DevelopmentLoopOutcome::Exhausted => Phase::Replan,
        })
    }

    fn apply_result(&self, graph: &mut TaskGraph, task_id: &str, result: &DevelopmentLoopResult) {
        match result.outcome {
            DevelopmentLoopOutcome::Verified => self.transition_task(
                graph,
                task_id,
                TaskStatus::Completed,
                "VERIFY",
                &format!("verified with {}", result.evidence_ref),
            ),
            DevelopmentLoopOutcome::Replanned => {
                if let Some(task) = graph.get_mut(task_id) {
                    task.retries += 1;
                }
                self.transition_task(
                    graph,
                    task_id,
                    TaskStatus::Ready,
                    "REPLAN",
                    &format!("replanned from {}", result.evidence_ref),
                );
            }
            DevelopmentLoopOutcome::Exhausted => self.transition_task(
                graph,
                task_id,
                TaskStatus::Failed,
                "REPLAN",
                &format!("attempt budget exhausted at {}", result.evidence_ref),
            ),
        }
    }

    fn transition_task(
        &self,
        graph: &mut TaskGraph,
        task_id: &str,
        next: TaskStatus,
        phase: &str,
        note: &str,
    ) {
        let from = match graph.get(task_id) {
            Some(task) => task.status,
            None => return,
        };
        graph.record(task_id, from, next, phase, note);
        if let Some(task) = graph.get_mut(task_id) {
            task.status = next;
            task.updated_at = Utc::now();
        }
    }

    /// Resume a task that was blocked waiting for approval.
    pub fn resume_approved(
        &mut self,
        graph: &mut TaskGraph,
        task_id: &str,
        approval_ref: &str,
    ) -> bool {
        if approval_ref.trim().is_empty() {
            return false;
        }
        let Some(envelope) = self.state.envelopes.get_mut(task_id) else {
            return false;
        };
        let Some(task) = graph.get(task_id) else {
            return false;
        };
        if task.status != TaskStatus::Blocked {
            return false;
        }

        envelope.policy.approval_ref = Some(approval_ref.to_string());
        self.transition_task(
            graph,
            task_id,
            TaskStatus::Ready,
            "AUTHORIZE",
            "approval supplied",
        );
        true
    }

    pub fn is_done(&self, graph: &TaskGraph) -> bool {
        graph.is_terminal() && graph.next_ready().is_none()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::tempdir;

    use super::*;
    use crate::{
        mock_verifier, ActionType, AgentAction, Capability, ContextRefs, EvidenceBinding,
        Lifecycle, PolicyBinding, RiskLevel, VerificationFabric, VerificationKind,
    };

    fn factory(max_attempts: u32, risk: RiskLevel) -> EnvelopeFactory {
        Box::new(move |task| ExecutionEnvelope {
            operation_id: format!("op-{}", task.id),
            run_id: "run-1".into(),
            task_id: task.id.clone(),
            action: AgentAction {
                id: format!("action-{}", task.id),
                task_id: task.id.clone(),
                agent_id: task.agent.clone().unwrap_or_else(|| "developer".into()),
                action_type: ActionType::WriteFile,
                reason: "implement task".into(),
                risk,
                capabilities: vec![Capability::WriteFile],
                payload: json!({
                    "path": format!("{}.txt", task.id),
                    "content": "done"
                }),
                expected: json!({}),
            },
            context: ContextRefs::default(),
            policy: PolicyBinding {
                risk,
                capabilities: vec![Capability::WriteFile],
                requires_approval: risk != RiskLevel::Low,
                approval_ref: None,
            },
            evidence: EvidenceBinding {
                required: vec!["unit_tests".into()],
                produced: vec![],
            },
            lifecycle: Lifecycle::new(max_attempts),
        })
    }

    fn graph() -> TaskGraph {
        let mut graph = TaskGraph::single("goal", "implement");
        graph.root = "task-1".into();
        let mut task = TaskNode::new("task-1", "task", "implement task");
        task.status = TaskStatus::Ready;
        graph.tasks.clear();
        graph.add(task);
        graph
    }

    fn assigner() -> Assigner {
        Assigner {
            assign: Box::new(|_| Some("developer".into())),
        }
    }

    fn decomposer() -> Decomposer {
        Decomposer {
            decompose: Box::new(|_| vec![]),
        }
    }

    fn context(path: &std::path::Path) -> VerificationContext {
        VerificationContext {
            workspace: path.to_string_lossy().into_owned(),
            changed: vec![],
        }
    }

    #[test]
    fn verified_attempt_completes_task_and_persists_evidence() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 1024 * 1024).unwrap();
        let fabric = VerificationFabric::new().with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, true),
        );
        let mut orchestrator = VerifiedOrchestrator::new(
            decomposer(),
            assigner(),
            DevelopmentLoop::new(fabric),
            factory(2, RiskLevel::Low),
        );
        let mut graph = graph();

        let phase = orchestrator
            .advance(&mut graph, &workspace, &context(dir.path()))
            .unwrap();

        assert_eq!(phase, Phase::Verify);
        assert_eq!(graph.get("task-1").unwrap().status, TaskStatus::Completed);
        let envelope = orchestrator.state.envelopes.get("task-1").unwrap();
        assert_eq!(envelope.evidence.produced.len(), 1);
        assert_eq!(envelope.lifecycle.attempt, 1);
    }

    #[test]
    fn failed_verification_reuses_envelope_and_then_exhausts() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 1024 * 1024).unwrap();
        let fabric = VerificationFabric::new().with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, false),
        );
        let mut orchestrator = VerifiedOrchestrator::new(
            decomposer(),
            assigner(),
            DevelopmentLoop::new(fabric),
            factory(2, RiskLevel::Low),
        );
        let mut graph = graph();

        let first = orchestrator
            .advance(&mut graph, &workspace, &context(dir.path()))
            .unwrap();
        assert_eq!(first, Phase::Replan);
        assert_eq!(graph.get("task-1").unwrap().status, TaskStatus::Ready);
        assert_eq!(orchestrator.state.envelopes["task-1"].lifecycle.attempt, 2);

        let second = orchestrator
            .advance(&mut graph, &workspace, &context(dir.path()))
            .unwrap();
        assert_eq!(second, Phase::Replan);
        assert_eq!(graph.get("task-1").unwrap().status, TaskStatus::Failed);
        let envelope = &orchestrator.state.envelopes["task-1"];
        assert_eq!(envelope.evidence.produced.len(), 2);
        assert_eq!(envelope.lifecycle.attempt, 2);
    }

    #[test]
    fn approval_blocks_and_resume_reuses_same_envelope() {
        let dir = tempdir().unwrap();
        let workspace = Workspace::new(dir.path(), 1024 * 1024).unwrap();
        let fabric = VerificationFabric::new().with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, true),
        );
        let mut orchestrator = VerifiedOrchestrator::new(
            decomposer(),
            assigner(),
            DevelopmentLoop::new(fabric),
            factory(2, RiskLevel::High),
        );
        let mut graph = graph();

        let phase = orchestrator
            .advance(&mut graph, &workspace, &context(dir.path()))
            .unwrap();
        assert_eq!(phase, Phase::Act);
        assert_eq!(graph.get("task-1").unwrap().status, TaskStatus::Blocked);
        assert!(orchestrator.state.envelopes["task-1"]
            .evidence
            .produced
            .is_empty());

        assert!(orchestrator.resume_approved(&mut graph, "task-1", "approval-1"));
        assert_eq!(graph.get("task-1").unwrap().status, TaskStatus::Ready);

        let phase = orchestrator
            .advance(&mut graph, &workspace, &context(dir.path()))
            .unwrap();
        assert_eq!(phase, Phase::Verify);
        assert_eq!(graph.get("task-1").unwrap().status, TaskStatus::Completed);
        assert_eq!(orchestrator.state.envelopes["task-1"].lifecycle.attempt, 1);
    }

    #[test]
    fn state_round_trips_for_recovery() {
        let mut state = VerifiedOrchestratorState::default();
        let mut task = TaskNode::new("task-1", "task", "x");
        task.agent = Some("developer".into());
        state
            .envelopes
            .insert("task-1".into(), (factory(2, RiskLevel::Low))(&task));

        let json = serde_json::to_string(&state).unwrap();
        let restored: VerifiedOrchestratorState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, state);
    }
}
