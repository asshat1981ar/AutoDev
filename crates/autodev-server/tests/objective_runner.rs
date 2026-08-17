use std::{
    fs,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use autodev_server::{
    ActionProposer, FileObjectiveStore, ObjectiveRunner, ObjectiveSnapshot, ObjectiveStatus,
    ObjectiveStore, ObjectiveView, RunnerError,
};
use forge_core::{
    ActionProposal, ActionType, AgentAction, Capability, PolicyDecision, RiskLevel, TaskGraph,
    TaskNode, TaskStatus, VerifiedOrchestratorState,
};
use tempfile::tempdir;
use tokio::sync::broadcast;

fn snapshot(id: &str) -> ObjectiveSnapshot {
    ObjectiveSnapshot {
        view: ObjectiveView {
            id: id.to_string(),
            repository: "asshat1981ar/AutoDev".to_string(),
            description: "Persist objective state".to_string(),
            branch: "autodev/persist-objective".to_string(),
            status: ObjectiveStatus::Running,
            current_task_id: Some("t-root".to_string()),
            current_phase: Some("act".to_string()),
            latest_evidence_ref: Some("evidence-1".to_string()),
            blocked_reason: None,
        },
        graph: TaskGraph::single("Persist objective state", "round-trip durable task state"),
        orchestrator: VerifiedOrchestratorState::default(),
    }
}

fn queued_snapshot(id: &str) -> ObjectiveSnapshot {
    ObjectiveSnapshot {
        view: ObjectiveView {
            id: id.to_string(),
            repository: "asshat1981ar/AutoDev".to_string(),
            description: "Plan one durable objective step".to_string(),
            branch: "autodev/plan-objective".to_string(),
            status: ObjectiveStatus::Queued,
            current_task_id: Some("t-root".to_string()),
            current_phase: None,
            latest_evidence_ref: None,
            blocked_reason: None,
        },
        graph: TaskGraph::single(
            "Plan objective",
            "advance the persisted graph exactly one planning step",
        ),
        orchestrator: VerifiedOrchestratorState::default(),
    }
}

struct CountingProposer {
    calls: AtomicUsize,
}

impl CountingProposer {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }
}

impl ActionProposer for CountingProposer {
    fn propose(&self, _task: &TaskNode) -> Result<ActionProposal, RunnerError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        panic!("planning preparation must not request an action proposal")
    }
}

struct ReadFileProposer {
    calls: AtomicUsize,
}

impl ReadFileProposer {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }
}

impl ActionProposer for ReadFileProposer {
    fn propose(&self, task: &TaskNode) -> Result<ActionProposal, RunnerError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ActionProposal {
            action: AgentAction {
                id: format!("proposal-{}", task.id),
                task_id: task.id.clone(),
                agent_id: "developer".to_string(),
                action_type: ActionType::ReadFile,
                reason: "inspect repository state".to_string(),
                risk: RiskLevel::Low,
                capabilities: vec![Capability::ReadFile],
                payload: serde_json::json!({"path": "README.md"}),
                expected: serde_json::json!({}),
            },
            decision: PolicyDecision::Allow,
            model: "fake-model".to_string(),
        })
    }
}

#[test]
fn file_store_round_trips_objective_snapshot_across_instances() {
    let directory = tempdir().expect("temp directory");
    let store = FileObjectiveStore::new(directory.path());
    let expected = snapshot("objective-1");

    store.put(&expected).expect("persist snapshot");
    assert!(!directory.path().join("objective-1.json.tmp").exists());

    let restarted = FileObjectiveStore::new(directory.path());
    assert_eq!(
        restarted.get("objective-1").expect("load snapshot"),
        Some(expected.clone())
    );
    assert_eq!(
        restarted.load_all().expect("load snapshots"),
        vec![expected]
    );
}

#[test]
fn file_store_ignores_stray_temporary_files() {
    let directory = tempdir().expect("temp directory");
    fs::write(
        directory.path().join("orphan.json.tmp"),
        br#"{"partial": true"#,
    )
    .expect("write partial file");

    let store = FileObjectiveStore::new(directory.path());
    assert!(store.load_all().expect("load snapshots").is_empty());
    assert_eq!(store.get("orphan").expect("load orphan"), None);
}

#[test]
fn runner_advances_queued_objective_to_persisted_planning_without_proposal() {
    let directory = tempdir().expect("temp directory");
    let store = Arc::new(FileObjectiveStore::new(directory.path()));
    store
        .put(&queued_snapshot("objective-plan-1"))
        .expect("persist queued objective");

    let proposer = Arc::new(CountingProposer::new());
    let (events, mut receiver) = broadcast::channel(8);
    let runner = ObjectiveRunner::new(Arc::clone(&store), Arc::clone(&proposer), events);

    let view = runner
        .advance_once("objective-plan-1")
        .expect("advance queued objective");

    assert_eq!(view.status, ObjectiveStatus::Planning);
    assert_eq!(view.current_task_id.as_deref(), Some("t-root"));
    assert_eq!(view.current_phase.as_deref(), Some("plan"));
    assert_eq!(proposer.calls.load(Ordering::SeqCst), 0);

    let persisted = store
        .get("objective-plan-1")
        .expect("load planned objective")
        .expect("planned objective exists");
    assert_eq!(persisted.view, view);
    assert_eq!(persisted.graph.root().status, TaskStatus::Planning);
    assert_eq!(persisted.graph.log.len(), 1);
    assert_eq!(persisted.graph.log[0].phase, "PLAN");

    let event = receiver.try_recv().expect("planning event");
    assert_eq!(event.event_type, "objective_planning");
    assert_eq!(event.objective_id, "objective-plan-1");
    assert_eq!(event.status, ObjectiveStatus::Planning);
    assert!(receiver.try_recv().is_err());
}

#[test]
fn runner_decomposes_planning_root_to_ready_without_proposal() {
    let directory = tempdir().expect("temp directory");
    let store = Arc::new(FileObjectiveStore::new(directory.path()));
    store
        .put(&queued_snapshot("objective-plan-2"))
        .expect("persist queued objective");

    let proposer = Arc::new(CountingProposer::new());
    let (events, mut receiver) = broadcast::channel(8);
    let runner = ObjectiveRunner::new(Arc::clone(&store), Arc::clone(&proposer), events);

    runner
        .advance_once("objective-plan-2")
        .expect("advance to planning");
    receiver.try_recv().expect("planning event");

    let view = runner
        .advance_once("objective-plan-2")
        .expect("decompose planning objective");

    assert_eq!(view.status, ObjectiveStatus::Planning);
    assert_eq!(view.current_task_id.as_deref(), Some("t-root"));
    assert_eq!(view.current_phase.as_deref(), Some("decompose"));
    assert_eq!(proposer.calls.load(Ordering::SeqCst), 0);

    let persisted = store
        .get("objective-plan-2")
        .expect("load decomposed objective")
        .expect("decomposed objective exists");
    assert_eq!(persisted.view, view);
    assert_eq!(persisted.graph.root().status, TaskStatus::Ready);
    assert_eq!(persisted.graph.log.len(), 2);
    assert_eq!(persisted.graph.log[1].phase, "DECOMPOSE");

    let event = receiver.try_recv().expect("decomposition lifecycle event");
    assert_eq!(event.event_type, "objective_planning");
    assert_eq!(event.phase.as_deref(), Some("decompose"));
    assert_eq!(event.status, ObjectiveStatus::Planning);
    assert!(receiver.try_recv().is_err());
}

#[test]
fn runner_persists_typed_action_proposal_for_ready_task() {
    let directory = tempdir().expect("temp directory");
    let store = Arc::new(FileObjectiveStore::new(directory.path()));
    store
        .put(&queued_snapshot("objective-propose-1"))
        .expect("persist queued objective");

    let proposer = Arc::new(ReadFileProposer::new());
    let (events, mut receiver) = broadcast::channel(8);
    let runner = ObjectiveRunner::new(Arc::clone(&store), Arc::clone(&proposer), events);

    runner
        .advance_once("objective-propose-1")
        .expect("advance to planning");
    receiver.try_recv().expect("planning event");
    runner
        .advance_once("objective-propose-1")
        .expect("advance to ready");
    receiver.try_recv().expect("decomposition event");

    let before = store
        .get("objective-propose-1")
        .expect("load ready objective")
        .expect("ready objective exists");
    assert_eq!(before.graph.root().status, TaskStatus::Ready);
    assert!(before.graph.root().planned_action.is_none());

    runner
        .advance_once("objective-propose-1")
        .expect("persist action proposal");

    assert_eq!(proposer.calls.load(Ordering::SeqCst), 1);
    let persisted = store
        .get("objective-propose-1")
        .expect("load proposed objective")
        .expect("proposed objective exists");
    let planned_action = persisted
        .graph
        .root()
        .planned_action
        .as_ref()
        .expect("serialized typed action");
    assert_eq!(planned_action["type"], "read_file");
    assert_eq!(planned_action["payload"]["path"], "README.md");
    assert!(planned_action.get("approval_ref").is_none());
    assert!(planned_action["payload"].get("approved").is_none());
    assert_eq!(persisted.graph.root().status, TaskStatus::Ready);
    assert!(receiver.try_recv().is_err());
}
