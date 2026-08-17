use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::Arc,
};

use forge_core::{
    ActionProposal, AgentAction, ContextRefs, Decomposer, EvidenceBinding, ExecutionEnvelope,
    Lifecycle, Planner, PolicyBinding, RiskLevel, TaskNode, TaskStatus,
};
use tokio::sync::broadcast;

use crate::{
    ObjectiveEvent, ObjectiveSnapshot, ObjectiveStatus, ObjectiveStore, ObjectiveView, StoreError,
};

pub trait ActionProposer: Send + Sync {
    fn propose(&self, task: &TaskNode) -> Result<ActionProposal, RunnerError>;
}

pub struct ObjectiveRunner<S: ObjectiveStore, P: ActionProposer> {
    store: Arc<S>,
    proposer: Arc<P>,
    events: broadcast::Sender<ObjectiveEvent>,
}

impl<S: ObjectiveStore, P: ActionProposer> ObjectiveRunner<S, P> {
    pub fn new(store: Arc<S>, proposer: Arc<P>, events: broadcast::Sender<ObjectiveEvent>) -> Self {
        Self {
            store,
            proposer,
            events,
        }
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
        self.store.put(snapshot)?;
        Ok(())
    }
}

fn execution_envelope_from_task(
    task: &TaskNode,
    run_id: &str,
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
            capabilities: action.capabilities.clone(),
            requires_approval,
            approval_ref: None,
        },
        action,
        context: ContextRefs::default(),
        evidence: EvidenceBinding::default(),
        lifecycle: Lifecycle::new(2),
    })
}

#[derive(Debug)]
pub enum RunnerError {
    Store(StoreError),
    Serialization(serde_json::Error),
    ObjectiveNotFound(String),
    TaskNotFound(String),
    MissingPlannedAction(String),
    NoReadyTask,
}

impl Display for RunnerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "objective runner store error: {error}"),
            Self::Serialization(error) => {
                write!(formatter, "objective runner serialization error: {error}")
            }
            Self::ObjectiveNotFound(id) => write!(formatter, "objective '{id}' not found"),
            Self::TaskNotFound(id) => write!(formatter, "task '{id}' not found"),
            Self::MissingPlannedAction(id) => {
                write!(formatter, "task '{id}' is missing a planned action")
            }
            Self::NoReadyTask => write!(formatter, "objective has no ready task"),
        }
    }
}

impl Error for RunnerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            Self::Serialization(error) => Some(error),
            Self::ObjectiveNotFound(_)
            | Self::TaskNotFound(_)
            | Self::MissingPlannedAction(_)
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

#[cfg(test)]
mod tests {
    use forge_core::{ActionType, Capability};
    use serde_json::json;

    use super::*;

    #[test]
    fn execution_envelope_deserializes_persisted_action_without_minting_approval() {
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

        let envelope = execution_envelope_from_task(&task, "run-1").expect("execution envelope");

        assert_eq!(envelope.action, action);
        assert_eq!(envelope.task_id, "task-1");
        assert_eq!(envelope.policy.risk, RiskLevel::High);
        assert_eq!(envelope.policy.capabilities, vec![Capability::WriteFile]);
        assert!(envelope.policy.requires_approval);
        assert_eq!(envelope.policy.approval_ref, None);
        assert_eq!(envelope.action.payload["approved"], true);
    }
}
