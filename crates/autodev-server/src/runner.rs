use std::{sync::Arc, thread, time::Duration};

use forge_core::{
    default_profiles, propose_action, ActionProposal, ActionProposalError, AgentAction, AgentProfile,
    AgentRole, Assigner, Capability, ContextRefs, Decomposer, DevelopmentLoop, EnvelopeError,
    EvidenceBinding, ExecutionEnvelope, Lifecycle, ModelProvider, Phase, Planner, PolicyBinding,
    PolicyDecision, Task, TaskNode, TaskStatus, VerificationContext, VerificationFabric,
    VerifiedOrchestrator, VerifiedOrchestratorError, Workspace,
};
use tokio::sync::broadcast;

use crate::{
    ObjectiveEvent, ObjectiveSnapshot, ObjectiveStatus, ObjectiveStore, ObjectiveView, StoreError,
};

pub trait ActionProposer: Send + Sync {
    fn propose(&self, task: &TaskNode) -> Result<ActionProposal, RunnerError>;
}

pub struct ModelActionProposer<P: ModelProvider + Send + Sync> {
    agent_id: String,
    profile: AgentProfile,
    provider: Arc<P>,
}

impl<P: ModelProvider + Send + Sync> ModelActionProposer<P> {
    pub fn new(agent_id: impl Into<String>, profile: AgentProfile, provider: Arc<P>) -> Self {
        Self {
            agent_id: agent_id.into(),
            profile,
            provider,
        }
    }
}

impl<P: ModelProvider + Send + Sync> ActionProposer for ModelActionProposer<P> {
    fn propose(&self, task: &TaskNode) -> Result<ActionProposal, RunnerError> {
        let runtime_task = Task {
            id: task.id.clone(),
            title: task.title.clone(),
            context: task.description.clone(),
        };
        Ok(propose_action(
            &self.agent_id,
            &self.profile,
            self.provider.as_ref(),
            &runtime_task,
        )?)
    }
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
    pub fn new(
        store: Arc<S>,
        proposer: Arc<P>,
        events: broadcast::Sender<ObjectiveEvent>,
    ) -> Self {
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

        if snapshot.view.status.is_terminal() || snapshot.view.status == ObjectiveStatus::Blocked {
            return Ok(snapshot.view);
        }

        if snapshot.graph.root().status == TaskStatus::Queued {
            Planner::default().plan(&mut snapshot.graph);
            snapshot.view.status = ObjectiveStatus::Planning;
            snapshot.view.current_task_id = Some(snapshot.graph.root.clone());
            snapshot.view.current_phase = Some("plan".to_string());
            snapshot.view.blocked_reason = None;
            return self.persist_and_emit(snapshot, "objective planning");
        }

        if snapshot.graph.root().status == TaskStatus::Planning {
            let root_id = snapshot.graph.root.clone();
            Decomposer {
                decompose: Box::new(|_| vec![]),
            }
            .decompose(&mut snapshot.graph, &root_id);
            snapshot.view.status = ObjectiveStatus::Planning;
            snapshot.view.current_task_id = Some(root_id);
            snapshot.view.current_phase = Some("decompose".to_string());
            snapshot.view.blocked_reason = None;
            return self.persist_and_emit(snapshot, "objective decomposed");
        }

        if snapshot
            .graph
            .next_ready()
            .is_some_and(|task| task.planned_action.is_none())
        {
            return self.persist_action_proposal(snapshot);
        }

        let Some(execution) = &self.execution else {
            return Ok(snapshot.view);
        };
        self.advance_verified(snapshot, execution)
    }

    pub fn resume_approved(
        &self,
        objective_id: &str,
        approval_ref: &str,
    ) -> Result<ObjectiveView, RunnerError> {
        if approval_ref.trim().is_empty() {
            return Err(RunnerError::ApprovalResumeFailed(objective_id.to_string()));
        }
        let mut snapshot = self
            .store
            .get(objective_id)?
            .ok_or_else(|| RunnerError::ObjectiveNotFound(objective_id.to_string()))?;
        let task_id = snapshot
            .view
            .current_task_id
            .clone()
            .ok_or_else(|| RunnerError::ApprovalResumeFailed(objective_id.to_string()))?;
        let envelope = snapshot
            .orchestrator
            .envelopes
            .get(&task_id)
            .cloned()
            .ok_or_else(|| RunnerError::ApprovalResumeFailed(objective_id.to_string()))?;
        let role = self
            .execution
            .as_ref()
            .map(|execution| execution.role)
            .unwrap_or(AgentRole::Developer);
        let mut orchestrator = VerifiedOrchestrator::new(
            Decomposer {
                decompose: Box::new(|_| vec![]),
            },
            Assigner {
                assign: Box::new(move |_| Some(role.as_str().to_string())),
            },
            DevelopmentLoop::new(VerificationFabric::new()),
            Box::new(move |_| envelope.clone()),
        );
        orchestrator.state = snapshot.orchestrator.clone();
        if !orchestrator.resume_approved(&mut snapshot.graph, &task_id, approval_ref) {
            return Err(RunnerError::ApprovalResumeFailed(objective_id.to_string()));
        }
        snapshot.orchestrator = orchestrator.state;
        snapshot.view.status = ObjectiveStatus::Running;
        snapshot.view.current_phase = Some("authorize".to_string());
        snapshot.view.blocked_reason = None;
        self.persist_and_emit(snapshot, "objective approval recorded")
    }

    pub fn pending_ids(&self) -> Result<Vec<String>, RunnerError> {
        Ok(self
            .store
            .load_all()?
            .into_iter()
            .filter(|snapshot| {
                !snapshot.view.status.is_terminal()
                    && snapshot.view.status != ObjectiveStatus::Blocked
            })
            .map(|snapshot| snapshot.view.id)
            .collect())
    }

    fn persist_action_proposal(
        &self,
        mut snapshot: ObjectiveSnapshot,
    ) -> Result<ObjectiveView, RunnerError> {
        let task = snapshot
            .graph
            .next_ready()
            .cloned()
            .ok_or(RunnerError::NoReadyTask)?;
        let proposal = self.proposer.propose(&task)?;

        if proposal.decision == PolicyDecision::Deny {
            transition_task(
                &mut snapshot,
                &task.id,
                TaskStatus::Failed,
                "POLICY",
                "proposed action denied by trusted agent profile",
            );
            snapshot.view.status = ObjectiveStatus::Failed;
            snapshot.view.current_task_id = Some(task.id);
            snapshot.view.current_phase = Some("policy".to_string());
            snapshot.view.blocked_reason = Some("proposed action denied by agent policy".to_string());
            return self.persist_and_emit(snapshot, "objective action denied");
        }

        let persisted_task = snapshot
            .graph
            .get_mut(&task.id)
            .ok_or_else(|| RunnerError::TaskNotFound(task.id.clone()))?;
        persisted_task.planned_action = Some(serde_json::to_value(&proposal.action)?);
        snapshot.view.status = ObjectiveStatus::Planning;
        snapshot.view.current_task_id = Some(task.id);
        snapshot.view.current_phase = Some("propose".to_string());
        snapshot.view.blocked_reason = None;
        self.persist_and_emit(snapshot, "objective action proposed")
    }

    fn advance_verified(
        &self,
        mut snapshot: ObjectiveSnapshot,
        execution: &RunnerExecution,
    ) -> Result<ObjectiveView, RunnerError> {
        let task = snapshot
            .graph
            .next_ready()
            .cloned()
            .ok_or(RunnerError::NoReadyTask)?;
        if task.planned_action.is_none() {
            return Err(RunnerError::MissingPlannedAction(task.id));
        }

        let profile = profile_for_role(execution.role)?;
        let capabilities = match trusted_capabilities_for_task(&task, &profile) {
            Ok(capabilities) => capabilities,
            Err(error) => {
                transition_task(
                    &mut snapshot,
                    &task.id,
                    TaskStatus::Failed,
                    "POLICY",
                    &error.to_string(),
                );
                snapshot.view.status = ObjectiveStatus::Failed;
                snapshot.view.current_task_id = Some(task.id);
                snapshot.view.current_phase = Some("policy".to_string());
                snapshot.view.blocked_reason = Some(error.to_string());
                return self.persist_and_emit(snapshot, "objective capability denied");
            }
        };
        let envelope = execution_envelope_from_task(
            &task,
            &snapshot.view.id,
            execution.role,
            capabilities,
            profile.policy.retry.max_attempts,
        )?;
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
            changed: changed_paths(&task),
        };
        let phase = orchestrator.advance(&mut snapshot.graph, &execution.workspace, &context)?;
        snapshot.orchestrator = orchestrator.state;
        update_view_from_verified_state(&mut snapshot, phase);
        let message = lifecycle_message(snapshot.view.status);
        self.persist_and_emit(snapshot, message)
    }

    fn persist_and_emit(
        &self,
        snapshot: ObjectiveSnapshot,
        message: &str,
    ) -> Result<ObjectiveView, RunnerError> {
        self.store.put(&snapshot)?;
        let _ = self
            .events
            .send(ObjectiveEvent::from_view(&snapshot.view, message));
        Ok(snapshot.view)
    }
}

pub fn run_objective_cycle<S, P>(runner: &ObjectiveRunner<S, P>) -> Result<usize, RunnerError>
where
    S: ObjectiveStore,
    P: ActionProposer,
{
    let ids = runner.pending_ids()?;
    let mut advanced = 0;
    for id in ids {
        runner.advance_once(&id)?;
        advanced += 1;
    }
    Ok(advanced)
}

pub fn run_objective_loop<S, P>(runner: ObjectiveRunner<S, P>, poll_interval: Duration)
where
    S: ObjectiveStore,
    P: ActionProposer,
{
    loop {
        if let Err(error) = run_objective_cycle(&runner) {
            eprintln!("objective worker cycle failed: {error}");
        }
        thread::sleep(poll_interval);
    }
}

fn profile_for_role(role: AgentRole) -> Result<AgentProfile, RunnerError> {
    default_profiles()
        .into_iter()
        .find(|profile| profile.role == role)
        .ok_or(RunnerError::AgentProfileNotFound(role))
}

fn trusted_capabilities_for_task(
    task: &TaskNode,
    profile: &AgentProfile,
) -> Result<Vec<Capability>, RunnerError> {
    let serialized = task
        .planned_action
        .as_ref()
        .ok_or_else(|| RunnerError::MissingPlannedAction(task.id.clone()))?;
    let action: AgentAction = serde_json::from_value(serialized.clone())?;
    match Capability::for_action(action.action_type) {
        Some(required) if profile.may(&required, action.risk) => Ok(vec![required]),
        Some(required) => Err(RunnerError::AgentCapabilityDenied {
            role: profile.role,
            capability: required,
            risk: action.risk,
        }),
        None => Ok(vec![]),
    }
}

fn execution_envelope_from_task(
    task: &TaskNode,
    run_id: &str,
    role: AgentRole,
    capabilities: Vec<Capability>,
    max_attempts: u32,
) -> Result<ExecutionEnvelope, RunnerError> {
    let serialized = task
        .planned_action
        .as_ref()
        .ok_or_else(|| RunnerError::MissingPlannedAction(task.id.clone()))?;
    let mut action: AgentAction = serde_json::from_value(serialized.clone())?;

    action.task_id = task.id.clone();
    action.agent_id = role.as_str().to_string();
    action.capabilities = capabilities.clone();
    let requires_approval = action.risk != forge_core::RiskLevel::Low;

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
        evidence: EvidenceBinding {
            required: vec!["unit_tests".to_string()],
            produced: vec![],
        },
        lifecycle: Lifecycle::new(max_attempts.max(1)),
    })
}

fn update_view_from_verified_state(snapshot: &mut ObjectiveSnapshot, phase: Phase) {
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
        (root.status == TaskStatus::Blocked).then(|| "waiting for trusted approval".to_string());
    snapshot.view.latest_evidence_ref = snapshot
        .orchestrator
        .envelopes
        .get(&root.id)
        .and_then(|envelope| envelope.evidence.produced.last().cloned());
}

fn changed_paths(task: &TaskNode) -> Vec<String> {
    task.planned_action
        .as_ref()
        .and_then(|value| value.get("payload"))
        .and_then(|payload| payload.get("path"))
        .and_then(|path| path.as_str())
        .map(|path| vec![path.to_string()])
        .unwrap_or_default()
}

fn transition_task(
    snapshot: &mut ObjectiveSnapshot,
    task_id: &str,
    next: TaskStatus,
    phase: &str,
    note: &str,
) {
    let Some(from) = snapshot.graph.get(task_id).map(|task| task.status) else {
        return;
    };
    snapshot.graph.record(task_id, from, next, phase, note);
    if let Some(task) = snapshot.graph.get_mut(task_id) {
        task.status = next;
        task.updated_at = chrono::Utc::now();
    }
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

#[derive(Debug, thiserror::Error)]
pub enum RunnerError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Envelope(#[from] EnvelopeError),
    #[error(transparent)]
    VerifiedOrchestrator(#[from] VerifiedOrchestratorError),
    #[error(transparent)]
    ActionProposal(#[from] ActionProposalError),
    #[error("objective '{0}' was not found")]
    ObjectiveNotFound(String),
    #[error("task '{0}' was not found")]
    TaskNotFound(String),
    #[error("task '{0}' is missing a planned action")]
    MissingPlannedAction(String),
    #[error("agent profile '{0:?}' was not found")]
    AgentProfileNotFound(AgentRole),
    #[error("agent profile '{role:?}' cannot grant {capability:?} at {risk:?} risk")]
    AgentCapabilityDenied {
        role: AgentRole,
        capability: Capability,
        risk: forge_core::RiskLevel,
    },
    #[error("objective has no ready task")]
    NoReadyTask,
    #[error("objective '{0}' cannot resume from approval")]
    ApprovalResumeFailed(String),
}
