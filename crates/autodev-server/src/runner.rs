use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::Arc,
};

use forge_core::{ActionProposal, Planner, TaskNode, TaskStatus};
use tokio::sync::broadcast;

use crate::{
    ObjectiveEvent, ObjectiveSnapshot, ObjectiveStatus, ObjectiveStore, ObjectiveView, StoreError,
};

pub trait ActionProposer: Send + Sync {
    fn propose(&self, task: &TaskNode) -> Result<ActionProposal, RunnerError>;
}

pub struct ObjectiveRunner<S: ObjectiveStore, P: ActionProposer> {
    store: Arc<S>,
    _proposer: Arc<P>,
    events: broadcast::Sender<ObjectiveEvent>,
}

impl<S: ObjectiveStore, P: ActionProposer> ObjectiveRunner<S, P> {
    pub fn new(store: Arc<S>, proposer: Arc<P>, events: broadcast::Sender<ObjectiveEvent>) -> Self {
        Self {
            store,
            _proposer: proposer,
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
}

#[derive(Debug)]
pub enum RunnerError {
    Store(StoreError),
    ObjectiveNotFound(String),
}

impl Display for RunnerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "objective runner store error: {error}"),
            Self::ObjectiveNotFound(id) => write!(formatter, "objective '{id}' not found"),
        }
    }
}

impl Error for RunnerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            Self::ObjectiveNotFound(_) => None,
        }
    }
}

impl From<StoreError> for RunnerError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}
