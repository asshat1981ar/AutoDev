use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::Arc,
};

use forge_core::{
    default_profiles, ActionProposal, AgentAction, AgentRole, Assigner, Capability, ContextRefs,
    Decomposer, DevelopmentLoop, EnvelopeError, EvidenceBinding, ExecutionEnvelope, Lifecycle,
    Phase, Planner, PolicyBinding, RiskLevel, TaskNode, TaskStatus, VerificationContext,
    VerificationFabric, VerifiedOrchestrator, VerifiedOrchestratorError, Workspace,
};
use tokio::sync::broadcast;

use crate::{
    ObjectiveEvent, ObjectiveSnapshot, ObjectiveStatus, ObjectiveStore, ObjectiveView, StoreError,
};

pub trait ActionProposer: Send + Sync {
    fn propose(&self, task: &TaskNode) -> Result<ActionProposal, RunnerError>;
}

pub type VerificationFactory = Arc<dyn Fn() -> VerificationFabric + Send + Sync>;

#[derive(Clone)]
pub struct RunnerExecution {
    workspace: Workspace,
    role: AgentRole,
    verification: VerificationFactory,
}

impl RunnerExecution {
    pub fn new(workspace: Workspace, role: AgentRole, verification: VerificationFactory) -> Self {
        Self {
            workspace,
            role,
            verification,
        }
    }
}

pub struct ObjectiveRunner<S: ObjectiveStore, P: ActionProposer> {
    store: Arc<S>,
    proposer: Arc<P>,
    events: broadcast::Sender<ObjectiveEvent>,
    execution: Option<RunnerExecution>,
}

impl<S: ObjectiveStore, P: ActionProposer> ObjectiveRunner<S, P> {
    pub fn new(store: Arc<S>, proposer: Arc<P>, events: broadcast::Sender<ObjectiveEvent>) -> Self {
        Self {
            store,
            proposer,
            events,
            execution: None,
        }
    }

    pub fn with_execution(mut self, execution: RunnerExecution) -> Self {
        self.execution = Some(execution);
        self
    }

    pub fn advance_once(&self, objective_id: &str) -> Result<ObjectiveView, RunnerError> {
        let mut snapshot = self
            .store
            .get(objective_id)?
            .ok_or_else(|| RunnerError::ObjectiveNotFound(objective_id.to_string()))?;

        if snapshot.graph.root().status == TaskStatus::Queued {
            self.advance_queued_to_planning(&mut snapshot)?;
        } else if snapshot.graph.root().status == TaskStatus::Planning {
            self.advance_planning_to_ready(&mut snapshot)?;
        } else if snapshot
            .graph
            .next_ready()
            .is_some_and(|task| task.planned_action.is_none())
        {
            self.persist_ready_action_proposal(&mut snapshot)?;
        } else if let Some(execution) = &self.execution {
            self.advance_verified(&mut snapshot, execution)?;
        }

        Ok(snapshot.view)
    }

    fn advance_queued_to_planning(
        &self,
        snapshot: &mut ObjectiveSnapshot,
    ) -> Result<(), RunnerError> {
        Planner::default().plan(&mut snapshot.graph);
        snapshot.view.status = ObjectiveStatus::Planning;
        snapshot.view.current_task_id = Some(snapshot.graph.root.clone());
        snapshot.view.current_phase = Some("plan".to_string());
        snapshot.view.blocked_reason = None;

        self.store.put(snapshot)?;
        let _ = self.events.send(ObjectiveEvent::from_view(
            &snapshot.view,
            "objective planning",
        ));
        Ok(())
    }

    fn advance_planning_to_ready(
        &self,
        snapshot: &mut ObjectiveSnapshot,
    ) -> Result<(), RunnerError> {
        let root_id = snapshot.graph.root.clone();
        let decomposer = Decomposer {
            decompose: Box::new(|_| vec![]),
        };
        decomposer.decompose(&mut snapshot.graph, &root_id);
        snapshot.view.status = ObjectiveStatus::Planning;
        snapshot.view.current_task_id = Some(root_id);
        snapshot.view.current_phase = Some("decompose".to_string());
        snapshot.view.blocked_reason = None;

        self.store.put(snapshot)?;
        let _ = self.events.send(ObjectiveEvent::from_view(
            &snapshot.view,
            "objective decomposed",
        ));
        Ok(())
    }

    fn persist_ready_action_proposal(
        &self,
        snapshot: &mut ObjectiveSnapshot,
    ) -> Result<(), RunnerError> {
        let task = snapshot
            .graph
            .next_ready()
            .cloned()
            .ok_or(RunnerError::NoReadyTask)?;
        let proposal = self.proposer.propose(&task)?;
        let serialized = serde_json::to_value(&proposal.action)?;
        let persisted_task = snapshot
            .graph
            .get_mut(&task.id)
            .ok_or_else(|| RunnerError::TaskNotFound(task.id.clone()))?;
        persisted_task.planned_action = Some(serialized);
        execution_envelope_from_task(persisted_task, &snapshot.view.id, vec![])?.validate()?;
        self.store.put(snapshot)?;
        Ok(())
    }

    fn advance_verified(
        &self,
        snapshot: &mut ObjectiveSnapshot,
        execution: &RunnerExecution,
    ) -> Result<(), RunnerError> {
        let task = snapshot
            .graph
            .next_ready()
            .cloned()
            .ok_or(RunnerError::NoReadyTask)?;
        if task.planned_action.is_none() {
            return Err(RunnerError::MissingPlannedAction(task.id));
        }

        let capabilities = trusted_capabilities_for_task(&task, execution.role)?;
        let envelope = execution_envelope_from_task(&task, &snapshot.view.id, capabilities)?;
        envelope.validate()?;

        let role = execution.role;
        let mut orchestrator = VerifiedOrchestrator::new(
            Decomposer {
                decompose: Box::new(|_| vec![]),
            },
            Assigner {
                assign: Box::new(move |_| Some(role.as_str().to_string())),
            },
            DevelopmentLoop::new((execution.verification)()),
            Box::new(move |_| envelope.clone()),
        );
        orchestrator.state = snapshot.orchestrator.clone();

        let context = VerificationContext {
            workspace: execution.workspace.root().to_string_lossy().into_owned(),
            changed: vec![],
        };
        let phase = orchestrator.advance(&mut snapshot.graph, &execution.workspace, &context)?;
        snapshot.orchestrator = orchestrator.state;
        self.update_view_from_verified_state(snapshot, phase);
        self.store.put(snapshot)?;
        let _ = self.events.send(ObjectiveEvent::from_view(
            &snapshot.view,
            lifecycle_message(snapshot.view.status),
        ));
        Ok(())
    }

    fn update_view_from_verified_state(&self, snapshot: &mut ObjectiveSnapshot, phase: Phase) {
        let root = snapshot.graph.root();
        snapshot.view.current_task_id = Some(root.id.clone());
        snapshot.view.current_phase = Some(
            match phase {
                Phase::Plan => "plan",
                Phase::Decompose => "decompose",
                Phase::Assign => "assign",
                Phase::Act => "act",
                Phase::Verify => "verify",
                Phase::Repair => "repair",
                Phase::Checkpoint => "checkpoint",
                Phase::Replan => "replan",
                Phase::Done => "done",
            }
            .to_string(),
        );
        snapshot.view.status = match root.status {
            TaskStatus::Queued => ObjectiveStatus::Queued,
            TaskStatus::Planning | TaskStatus::Ready => match phase {
                Phase::Replan => ObjectiveStatus::Replanned,
                _ => ObjectiveStatus::Planning,
            },
            TaskStatus::Running => ObjectiveStatus::Running,
            TaskStatus::Verifying => ObjectiveStatus::Verifying,
            TaskStatus::Repairing => ObjectiveStatus::Replanned,
            TaskStatus::Blocked => ObjectiveStatus::Blocked,
            TaskStatus::Completed => ObjectiveStatus::Completed,
            TaskStatus::Failed | TaskStatus::Cancelled => ObjectiveStatus::Failed,
        };
        snapshot.view.blocked_reason =
            (root.status == TaskStatus::Blocked).then(|| "waiting for approval".to_string());
        snapshot.view.latest_evidence_ref = snapshot
            .orchestrator
            .envelopes
            .get(&root.id)
            .and_then(|envelope| envelope.evidence.produced.last().cloned());
    }
}

fn trusted_capabilities_for_task(
    task: &TaskNode,
    role: AgentRole,
) -> Result<Vec<Capability>, RunnerError> {
    let serialized = task
        .planned_action
        .as_ref()
        .ok_or_else(|| RunnerError::MissingPlannedAction(task.id.clone()))?;
    let action: AgentAction = serde_json::from_value(serialized.clone())?;
    let profile = default_profiles()
        .into_iter()
        .find(|profile| profile.role == role)
        .ok_or(RunnerError::AgentProfileNotFound(role))?;

    match Capability::for_action(action.action_type) {
        Some(required) if profile.may(&required, action.risk) => Ok(vec![required]),
        Some(required) => Err(RunnerError::AgentCapabilityDenied {
            role,
            capability: required,
            risk: action.risk,
        }),
        None => Ok(vec![]),
    }
}

fn execution_envelope_from_task(
    task: &TaskNode,
    run_id: &str,
    capabilities: Vec<Capability>,
) -> Result<ExecutionEnvelope, RunnerError> {
    let serialized = task
        .planned_action
        .as_ref()
        .ok_or_else(|| RunnerError::MissingPlannedAction(task.id.clone()))?;
    let action: AgentAction = serde_json::from_value(serialized.clone())?;
    let requires_approval = !matches!(action.risk, RiskLevel::Low);

    Ok(ExecutionEnvelope {
        operation_id: format!("op-{run_id}-{}", task.id),
        run_id: run_id.to_string(),
        task_id: task.id.clone(),
        policy: PolicyBinding {
            risk: action.risk,
            capabilities,
            requires_approval,
            approval_ref: None,
        },
        action,
        context: ContextRefs::default(),
        evidence: EvidenceBinding::default(),
        lifecycle: Lifecycle::new(2),
    })
}

fn lifecycle_message(status: ObjectiveStatus) -> &'static str {
    match status {
        ObjectiveStatus::Queued => "objective queued",
        ObjectiveStatus::Planning => "objective planning",
        ObjectiveStatus::Running => "objective running",
        ObjectiveStatus::Blocked => "objective blocked",
        ObjectiveStatus::Verifying => "objective verifying",
        ObjectiveStatus::Replanned => "objective replanned",
        ObjectiveStatus::Completed => "objective completed",
        ObjectiveStatus::Failed => "objective failed",
    }
}

#[derive(Debug)]
pub enum RunnerError {
    Store(StoreError),
    Serialization(serde_json::Error),
    Envelope(EnvelopeError),
    VerifiedOrchestrator(VerifiedOrchestratorError),
    ObjectiveNotFound(String),
    TaskNotFound(String),
    MissingPlannedAction(String),
    AgentProfileNotFound(AgentRole),
    AgentCapabilityDenied {
        role: AgentRole,
        capability: Capability,
        risk: RiskLevel,
    },
    NoReadyTask,
}

impl Display for RunnerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "objective runner store error: {error}"),
            Self::Serialization(error) => {
                write!(formatter, "objective runner serialization error: {error}")
            }
            Self::Envelope(error) => write!(formatter, "objective runner envelope error: {error}"),
            Self::VerifiedOrchestrator(error) => {
                write!(formatter, "objective runner orchestration error: {error}")
            }
            Self::ObjectiveNotFound(id) => write!(formatter, "objective '{id}' not found"),
            Self::TaskNotFound(id) => write!(formatter, "task '{id}' not found"),
            Self::MissingPlannedAction(id) => {
                write!(formatter, "task '{id}' is missing a planned action")
            }
            Self::AgentProfileNotFound(role) => {
                write!(formatter, "agent profile '{}' was not found", role.as_str())
            }
            Self::AgentCapabilityDenied {
                role,
                capability,
                risk,
            } => write!(
                formatter,
                "agent profile '{}' cannot grant {:?} at {:?} risk",
                role.as_str(),
                capability,
                risk
            ),
            Self::NoReadyTask => write!(formatter, "objective has no ready task"),
        }
    }
}

impl Error for RunnerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            Self::Serialization(error) => Some(error),
            Self::Envelope(error) => Some(error),
            Self::VerifiedOrchestrator(error) => Some(error),
            Self::ObjectiveNotFound(_)
            | Self::TaskNotFound(_)
            | Self::MissingPlannedAction(_)
            | Self::AgentProfileNotFound(_)
            | Self::AgentCapabilityDenied { .. }
            | Self::NoReadyTask => None,
        }
    }
}

impl From<StoreError> for RunnerError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<serde_json::Error> for RunnerError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

impl From<EnvelopeError> for RunnerError {
    fn from(error: EnvelopeError) -> Self {
        Self::Envelope(error)
    }
}

impl From<VerifiedOrchestratorError> for RunnerError {
    fn from(error: VerifiedOrchestratorError) -> Self {
        Self::VerifiedOrchestrator(error)
    }
}

#[cfg(test)]
mod tests {
    use forge_core::{ActionType, Capability};
    use serde_json::json;

    use super::*;

    #[test]
    fn execution_envelope_does_not_promote_requested_capabilities_or_payload_approval() {
        let action = AgentAction {
            id: "action-1".to_string(),
            task_id: "task-1".to_string(),
            agent_id: "developer".to_string(),
            action_type: ActionType::WriteFile,
            reason: "change one file".to_string(),
            risk: RiskLevel::High,
            capabilities: vec![Capability::WriteFile],
            payload: json!({
                "path": "src/lib.rs",
                "content": "x",
                "approved": true
            }),
            expected: json!({}),
        };
        let mut task = TaskNode::new("task-1", "change", "change one file");
        task.planned_action = Some(serde_json::to_value(&action).expect("serialize action"));

        let envelope =
            execution_envelope_from_task(&task, "run-1", vec![]).expect("execution envelope");

        assert_eq!(envelope.action, action);
        assert_eq!(envelope.task_id, "task-1");
        assert_eq!(envelope.policy.risk, RiskLevel::High);
        assert!(envelope.policy.capabilities.is_empty());
        assert!(envelope.policy.requires_approval);
        assert_eq!(envelope.policy.approval_ref, None);
        assert_eq!(envelope.action.payload["approved"], true);
    }
}
