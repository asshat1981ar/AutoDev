use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::Arc,
};

use forge_core::{ActionProposal, Decomposer, Planner, TaskNode, TaskStatus};
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

#[derive(Debug)]
pub enum RunnerError {
    Store(StoreError),
    Serialization(serde_json::Error),
    ObjectiveNotFound(String),
    TaskNotFound(String),
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
            Self::NoReadyTask => write!(formatter, "objective has no ready task"),
        }
    }
}

impl Error for RunnerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            Self::Serialization(error) => Some(error),
            Self::ObjectiveNotFound(_) | Self::TaskNotFound(_) | Self::NoReadyTask => None,
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
