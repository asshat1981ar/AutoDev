use std::sync::Arc;

use autodev_server::{
    ActionProposer, FileObjectiveStore, InMemoryObjectiveStore, ObjectiveApprovalGrant,
    ObjectiveEvent, ObjectiveRunner, ObjectiveSnapshot, ObjectiveStatus, ObjectiveStore,
    ObjectiveView, RunnerError, RunnerExecution,
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
            description: "review hardening".into(),
            branch: "autodev/review-hardening".into(),
            status: ObjectiveStatus::Queued,
            current_task_id: Some("t-root".into()),
            current_phase: None,
            latest_evidence_ref: None,
            blocked_reason: None,
        },
        graph: TaskGraph::single("objective", "review hardening"),
        orchestrator: VerifiedOrchestratorState::default(),
        evidence: vec![],
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
fn custom_proposer_cannot_downgrade_write_to_low_risk() {
    let workspace_dir = tempdir().unwrap();
    let workspace = Workspace::new(workspace_dir.path(), 1024 * 1024).unwrap();
    let store = Arc::new(InMemoryObjectiveStore::default());
    store.put(&snapshot("objective-1")).unwrap();
    let proposer = Arc::new(FixedProposer {
        proposal: proposal(
            ActionType::WriteFile,
            RiskLevel::Low,
            json!({"path": "marker.txt", "content": "must not write"}),
        ),
    });
    let runner = ObjectiveRunner::new(store.clone(), proposer, events())
        .with_execution(execution(workspace, true));

    advance_to_proposed(&runner, "objective-1");
    let blocked = runner.advance_once("objective-1").unwrap();

    assert_eq!(blocked.status, ObjectiveStatus::Blocked);
    assert!(!workspace_dir.path().join("marker.txt").exists());
    let persisted = store.get("objective-1").unwrap().unwrap();
    assert_eq!(
        persisted.orchestrator.envelopes["t-root"].action.risk,
        RiskLevel::Medium
    );
}

#[test]
fn evidence_reference_resolves_from_persisted_snapshot_after_restart() {
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
    let runner =
        ObjectiveRunner::new(store, proposer, events()).with_execution(execution(workspace, true));

    advance_to_proposed(&runner, "objective-1");
    let completed = runner.advance_once("objective-1").unwrap();
    let evidence_ref = completed.latest_evidence_ref.expect("evidence ref");
    drop(runner);

    let reopened = FileObjectiveStore::open(state_dir.path()).unwrap();
    let restored = reopened.get("objective-1").unwrap().unwrap();
    let evidence = restored
        .evidence
        .iter()
        .find(|evidence| evidence.record.id == evidence_ref)
        .expect("durable evidence");
    assert!(evidence.verify());
    assert_eq!(evidence.record.task_id, "t-root");
}

#[test]
fn approval_grant_must_match_objective_and_task_scope() {
    let workspace_dir = tempdir().unwrap();
    let workspace = Workspace::new(workspace_dir.path(), 1024 * 1024).unwrap();
    let store = Arc::new(InMemoryObjectiveStore::default());
    store.put(&snapshot("objective-1")).unwrap();
    let proposer = Arc::new(FixedProposer {
        proposal: proposal(
            ActionType::WriteFile,
            RiskLevel::Medium,
            json!({"path": "marker.txt", "content": "done"}),
        ),
    });
    let runner = ObjectiveRunner::new(store.clone(), proposer, events())
        .with_execution(execution(workspace, true));

    advance_to_proposed(&runner, "objective-1");
    assert_eq!(
        runner.advance_once("objective-1").unwrap().status,
        ObjectiveStatus::Blocked
    );

    let wrong_objective =
        ObjectiveApprovalGrant::new("objective-2", "t-root", "approval-1").unwrap();
    assert!(runner.resume_approved(&wrong_objective).is_err());

    let wrong_task =
        ObjectiveApprovalGrant::new("objective-1", "other-task", "approval-1").unwrap();
    assert!(runner.resume_approved(&wrong_task).is_err());

    assert!(ObjectiveApprovalGrant::new("objective-1", "t-root", "   ").is_err());
    assert_eq!(
        store.get("objective-1").unwrap().unwrap().view.status,
        ObjectiveStatus::Blocked
    );
    assert!(!workspace_dir.path().join("marker.txt").exists());
}

#[test]
fn matching_scoped_approval_resumes_existing_envelope() {
    let workspace_dir = tempdir().unwrap();
    let workspace = Workspace::new(workspace_dir.path(), 1024 * 1024).unwrap();
    let store = Arc::new(InMemoryObjectiveStore::default());
    store.put(&snapshot("objective-1")).unwrap();
    let proposer = Arc::new(FixedProposer {
        proposal: proposal(
            ActionType::WriteFile,
            RiskLevel::Medium,
            json!({"path": "marker.txt", "content": "done"}),
        ),
    });
    let runner =
        ObjectiveRunner::new(store, proposer, events()).with_execution(execution(workspace, true));

    advance_to_proposed(&runner, "objective-1");
    assert_eq!(
        runner.advance_once("objective-1").unwrap().status,
        ObjectiveStatus::Blocked
    );

    let grant = ObjectiveApprovalGrant::new("objective-1", "t-root", "approval-1").unwrap();
    assert_eq!(
        runner.resume_approved(&grant).unwrap().status,
        ObjectiveStatus::Running
    );
    assert_eq!(
        runner.advance_once("objective-1").unwrap().status,
        ObjectiveStatus::Completed
    );
    assert_eq!(
        std::fs::read_to_string(workspace_dir.path().join("marker.txt")).unwrap(),
        "done"
    );
}
