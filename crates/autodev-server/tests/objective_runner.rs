use std::sync::Arc;

use autodev_server::{
    ActionProposer, FileObjectiveStore, InMemoryObjectiveStore, ObjectiveEvent, ObjectiveRunner,
    ObjectiveSnapshot, ObjectiveStatus, ObjectiveStore, ObjectiveView, RunnerError,
    RunnerExecution,
};
use forge_core::{
    mock_verifier, ActionProposal, ActionType, AgentAction, AgentRole, Capability, PolicyDecision,
    RiskLevel, TaskGraph, VerificationFabric, VerificationKind, VerifiedOrchestratorState,
    Workspace,
};
use serde_json::json;
use tempfile::tempdir;
use tokio::sync::broadcast;

#[derive(Clone)]
struct FixedProposer {
    proposal: ActionProposal,
}

impl ActionProposer for FixedProposer {
    fn propose(&self, _task: &forge_core::TaskNode) -> Result<ActionProposal, RunnerError> {
        Ok(self.proposal.clone())
    }
}

fn proposal(
    action_type: ActionType,
    risk: RiskLevel,
    payload: serde_json::Value,
) -> ActionProposal {
    ActionProposal {
        action: AgentAction {
            id: "action-1".into(),
            task_id: "t-root".into(),
            agent_id: "model-supplied-agent".into(),
            action_type,
            reason: "perform objective".into(),
            risk,
            capabilities: vec![Capability::ApprovalCritical],
            payload,
            expected: json!({}),
        },
        decision: match risk {
            RiskLevel::Low => PolicyDecision::Allow,
            RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical => {
                PolicyDecision::RequireApproval
            }
        },
        model: "test-model".into(),
    }
}

fn snapshot(id: &str) -> ObjectiveSnapshot {
    ObjectiveSnapshot {
        view: ObjectiveView {
            id: id.into(),
            repository: "asshat1981ar/AutoDev".into(),
            description: "exercise verified objective path".into(),
            branch: "autodev/test".into(),
            status: ObjectiveStatus::Queued,
            current_task_id: Some("t-root".into()),
            current_phase: None,
            latest_evidence_ref: None,
            blocked_reason: None,
        },
        graph: TaskGraph::single("objective", "exercise verified objective path"),
        orchestrator: VerifiedOrchestratorState::default(),
    }
}

fn events() -> broadcast::Sender<ObjectiveEvent> {
    broadcast::channel(64).0
}

fn execution(workspace: Workspace, verification_passes: bool) -> RunnerExecution {
    RunnerExecution::new(
        workspace,
        AgentRole::Developer,
        Arc::new(move || {
            VerificationFabric::new().with(
                VerificationKind::UnitTests,
                mock_verifier(VerificationKind::UnitTests, verification_passes),
            )
        }),
    )
}

fn advance_to_proposed<S: ObjectiveStore>(runner: &ObjectiveRunner<S, FixedProposer>, id: &str) {
    assert_eq!(
        runner.advance_once(id).unwrap().status,
        ObjectiveStatus::Planning
    );
    assert_eq!(
        runner.advance_once(id).unwrap().status,
        ObjectiveStatus::Planning
    );
    assert_eq!(
        runner.advance_once(id).unwrap().status,
        ObjectiveStatus::Planning
    );
}

#[test]
fn file_store_round_trips_state_and_ignores_temp_files() {
    let dir = tempdir().unwrap();
    let store = FileObjectiveStore::open(dir.path()).unwrap();
    let mut state = snapshot("objective-1");
    state.view.status = ObjectiveStatus::Replanned;
    state.view.current_phase = Some("replan".into());
    store.put(&state).unwrap();
    std::fs::write(dir.path().join(".interrupted.json.tmp"), b"not-json").unwrap();

    let restored = FileObjectiveStore::open(dir.path()).unwrap();
    assert_eq!(restored.get("objective-1").unwrap(), Some(state));
    assert_eq!(restored.load_all().unwrap().len(), 1);
}

#[test]
fn corrupt_persisted_snapshot_fails_store_open() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("objective-1.json"), b"{broken").unwrap();
    assert!(FileObjectiveStore::open(dir.path()).is_err());
}

#[test]
fn low_risk_read_executes_through_verified_orchestrator_with_trusted_capabilities() {
    let workspace_dir = tempdir().unwrap();
    std::fs::write(workspace_dir.path().join("README.md"), "hello").unwrap();
    let workspace = Workspace::new(workspace_dir.path(), 1024 * 1024).unwrap();
    let store = Arc::new(InMemoryObjectiveStore::default());
    store.put(&snapshot("objective-1")).unwrap();
    let proposer = Arc::new(FixedProposer {
        proposal: proposal(
            ActionType::ReadFile,
            RiskLevel::Low,
            json!({"path": "README.md", "approved": true}),
        ),
    });
    let runner = ObjectiveRunner::new(store.clone(), proposer, events())
        .with_execution(execution(workspace, true));

    advance_to_proposed(&runner, "objective-1");
    let result = runner.advance_once("objective-1").unwrap();

    assert_eq!(result.status, ObjectiveStatus::Completed);
    assert!(result.latest_evidence_ref.is_some());
    let restored = store.get("objective-1").unwrap().unwrap();
    let envelope = &restored.orchestrator.envelopes["t-root"];
    assert_eq!(envelope.action.capabilities, vec![Capability::ReadFile]);
    assert_eq!(envelope.policy.capabilities, vec![Capability::ReadFile]);
    assert_eq!(envelope.policy.approval_ref, None);
    assert_eq!(envelope.action.payload["approved"], true);
}

#[test]
fn medium_risk_write_blocks_without_trusted_approval_and_can_resume_internally() {
    let workspace_dir = tempdir().unwrap();
    let workspace = Workspace::new(workspace_dir.path(), 1024 * 1024).unwrap();
    let store = Arc::new(InMemoryObjectiveStore::default());
    store.put(&snapshot("objective-1")).unwrap();
    let proposer = Arc::new(FixedProposer {
        proposal: proposal(
            ActionType::WriteFile,
            RiskLevel::Medium,
            json!({"path": "marker.txt", "content": "done", "approved": true}),
        ),
    });
    let runner = ObjectiveRunner::new(store.clone(), proposer, events())
        .with_execution(execution(workspace, true));

    advance_to_proposed(&runner, "objective-1");
    let blocked = runner.advance_once("objective-1").unwrap();
    assert_eq!(blocked.status, ObjectiveStatus::Blocked);
    assert!(!workspace_dir.path().join("marker.txt").exists());
    let envelope = &store
        .get("objective-1")
        .unwrap()
        .unwrap()
        .orchestrator
        .envelopes["t-root"];
    assert_eq!(envelope.policy.approval_ref, None);
    assert_eq!(envelope.action.capabilities, vec![Capability::WriteFile]);

    let resumed = runner
        .resume_approved("objective-1", "trusted-approval-1")
        .unwrap();
    assert_eq!(resumed.status, ObjectiveStatus::Running);
    let completed = runner.advance_once("objective-1").unwrap();
    assert_eq!(completed.status, ObjectiveStatus::Completed);
    assert_eq!(
        std::fs::read_to_string(workspace_dir.path().join("marker.txt")).unwrap(),
        "done"
    );
}

#[test]
fn verification_rejection_replans_then_exhausts_without_resetting_envelope() {
    let workspace_dir = tempdir().unwrap();
    std::fs::write(workspace_dir.path().join("README.md"), "hello").unwrap();
    let workspace = Workspace::new(workspace_dir.path(), 1024 * 1024).unwrap();
    let store = Arc::new(InMemoryObjectiveStore::default());
    store.put(&snapshot("objective-1")).unwrap();
    let proposer = Arc::new(FixedProposer {
        proposal: proposal(
            ActionType::ReadFile,
            RiskLevel::Low,
            json!({"path": "README.md"}),
        ),
    });
    let runner = ObjectiveRunner::new(store.clone(), proposer, events())
        .with_execution(execution(workspace, false));

    advance_to_proposed(&runner, "objective-1");
    let first = runner.advance_once("objective-1").unwrap();
    assert_eq!(first.status, ObjectiveStatus::Replanned);
    assert_eq!(
        store
            .get("objective-1")
            .unwrap()
            .unwrap()
            .orchestrator
            .envelopes["t-root"]
            .lifecycle
            .attempt,
        2
    );

    let second = runner.advance_once("objective-1").unwrap();
    assert_eq!(second.status, ObjectiveStatus::Replanned);
    assert_eq!(
        store
            .get("objective-1")
            .unwrap()
            .unwrap()
            .orchestrator
            .envelopes["t-root"]
            .lifecycle
            .attempt,
        3
    );

    let third = runner.advance_once("objective-1").unwrap();
    assert_eq!(third.status, ObjectiveStatus::Failed);
    let persisted = store.get("objective-1").unwrap().unwrap();
    let envelope = &persisted.orchestrator.envelopes["t-root"];
    assert_eq!(envelope.lifecycle.attempt, 3);
    assert_eq!(envelope.evidence.produced.len(), 3);
}

#[test]
fn file_backed_restart_resumes_persisted_attempt_budget() {
    let state_dir = tempdir().unwrap();
    let workspace_dir = tempdir().unwrap();
    std::fs::write(workspace_dir.path().join("README.md"), "hello").unwrap();
    let workspace = Workspace::new(workspace_dir.path(), 1024 * 1024).unwrap();

    let store = Arc::new(FileObjectiveStore::open(state_dir.path()).unwrap());
    store.put(&snapshot("objective-1")).unwrap();
    let proposer = Arc::new(FixedProposer {
        proposal: proposal(
            ActionType::ReadFile,
            RiskLevel::Low,
            json!({"path": "README.md"}),
        ),
    });
    let runner = ObjectiveRunner::new(store, proposer.clone(), events())
        .with_execution(execution(workspace.clone(), false));
    advance_to_proposed(&runner, "objective-1");
    assert_eq!(
        runner.advance_once("objective-1").unwrap().status,
        ObjectiveStatus::Replanned
    );
    drop(runner);

    let reopened = Arc::new(FileObjectiveStore::open(state_dir.path()).unwrap());
    let resumed = ObjectiveRunner::new(reopened.clone(), proposer, events())
        .with_execution(execution(workspace, false));
    assert_eq!(
        resumed.advance_once("objective-1").unwrap().status,
        ObjectiveStatus::Replanned
    );
    assert_eq!(
        reopened
            .get("objective-1")
            .unwrap()
            .unwrap()
            .orchestrator
            .envelopes["t-root"]
            .lifecycle
            .attempt,
        3
    );
    assert_eq!(
        resumed.advance_once("objective-1").unwrap().status,
        ObjectiveStatus::Failed
    );
    let persisted = reopened.get("objective-1").unwrap().unwrap();
    let envelope = &persisted.orchestrator.envelopes["t-root"];
    assert_eq!(envelope.lifecycle.attempt, 3);
    assert_eq!(envelope.evidence.produced.len(), 3);
}
