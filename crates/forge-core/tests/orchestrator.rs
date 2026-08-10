//! Integration tests for the SDLC orchestrator driving a real, durable loop.

use forge_core as fc;
use forge_core::{
    Assigner, Decomposer, ExecResult, Orchestrator, Planner, TaskExecutor, TaskGraph, TaskStatus,
    Verdict, Verifier,
};
use serde_json::json;

/// A deterministic orchestrator that writes a file as the "act" step.
fn orchestrator_for_write() -> Orchestrator {
    let ws = fc::Workspace::new(".", 1 << 20).unwrap();
    Orchestrator::new(
        Planner::default(),
        Decomposer {
            decompose: Box::new(|t| {
                vec![(
                    format!("{}-impl", t.id),
                    format!("{} implementation", t.title),
                    "implement".to_string(),
                )]
            }),
        },
        Assigner {
            assign: Box::new(|_| Some("developer".to_string())),
        },
        TaskExecutor {
            run: Box::new(move |task| {
                // Deterministic action: write a marker file named after the task.
                let path = format!("{}.out", task.id);
                let action = fc::AgentAction {
                    id: format!("act-{}", task.id),
                    task_id: task.id.clone(),
                    agent_id: "developer".to_string(),
                    action_type: fc::ActionType::WriteFile,
                    reason: "implement".to_string(),
                    risk: fc::RiskLevel::Low,
                    capabilities: vec![fc::Capability::WriteFile],
                    payload: json!({ "path": path, "content": "done" }),
                    expected: json!({}),
                };
                let res = fc::execute(&fc::ExecutableAction::new(action, ws.clone()));
                match res {
                    Ok(r) => ExecResult {
                        ok: r.status == fc::ExecutionStatus::Succeeded,
                        note: "wrote".to_string(),
                        payload: r.verification.unwrap_or(json!({})),
                    },
                    Err(e) => ExecResult {
                        ok: false,
                        note: e.to_string(),
                        payload: json!({}),
                    },
                }
            }),
        },
        Verifier {
            check: Box::new(|_t, r| if r.ok { Verdict::Pass } else { Verdict::Fail }),
        },
        fc::default_repairer(),
    )
}

#[test]
fn loop_runs_to_terminal_with_durable_tasks() {
    let mut graph = TaskGraph::single("implement feature", "add a marker file");
    let mut orch = orchestrator_for_write();
    orch.plan(&mut graph);

    let mut steps = 0;
    while !orch.is_done(&graph) && steps < 20 {
        orch.advance(&mut graph);
        steps += 1;
    }
    assert!(orch.is_done(&graph), "loop did not terminate");
    assert!(graph.checkpoint.is_some(), "checkpoint taken for recovery");
    assert!(!graph.log.is_empty(), "transition log populated");

    // Every task reached a terminal status.
    for t in graph.tasks.values() {
        assert!(
            matches!(
                t.status,
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
            ),
            "task {} not terminal: {:?}",
            t.id,
            t.status
        );
    }
}

#[test]
fn graph_is_recoverable_from_snapshot() {
    let mut graph = TaskGraph::single("goal", "x");
    let mut orch = orchestrator_for_write();
    orch.plan(&mut graph);
    orch.advance(&mut graph); // decompose
    orch.advance(&mut graph); // act root

    // Snapshot == checkpoint; deserialize back to a full graph.
    let snapshot = graph.checkpoint.clone().unwrap();
    let restored: std::collections::BTreeMap<String, fc::TaskNode> =
        serde_json::from_value(snapshot).unwrap();
    assert!(restored.contains_key("t-root"));
    assert_eq!(restored.len(), graph.tasks.len());
}
