//! The SDLC orchestrator: the first autonomous development loop.
//!
//! The loop is `PLAN → DECOMPOSE → ASSIGN → ACT → VERIFY → REPAIR → CHECKPOINT
//! → REPLAN`, built around **durable tasks** (a [`TaskGraph`]) rather than
//! conversational messages.
//!
//! Design goals (per the task):
//!
//! - **Observability**: every task transition is recorded in a [`TransitionLog`].
//! - **Recoverability**: the whole graph is serializable, so state can be
//!   snapshotted (checkpoint) and restored.
//! - **Deterministic state**: each phase is a pure, testable transition on the
//!   graph.
//! - **Clear task transitions**: tasks move through explicit [`TaskStatus`]es.
//! - **Human intervention**: a task can be blocked awaiting human approval and
//!   resumed explicitly.
//!
//! The orchestrator does **not** maximize autonomy: it favors a small, auditable,
//! single-step loop over aggressive autonomous churn.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The lifecycle status of a durable task.
///
/// Mirrors `task.schema.json` plus a `repairing` state for the REPAIR phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    Planning,
    Ready,
    Running,
    Verifying,
    Repairing,
    Blocked,
    Completed,
    Failed,
    Cancelled,
}

/// A durable unit of work in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    /// Priority 0..100.
    pub priority: u32,
    /// Ids of tasks that must complete first.
    pub dependencies: Vec<String>,
    /// Acceptance criteria used by the verifier.
    pub acceptance_criteria: Vec<String>,
    /// The agent role assigned (if any).
    pub agent: Option<String>,
    /// The action payload produced by the plan (type + data).
    pub planned_action: Option<serde_json::Value>,
    /// Number of repair/retry attempts so far.
    pub retries: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaskNode {
    /// Create a new queued task.
    pub fn new(id: &str, title: &str, description: &str) -> Self {
        let now = Utc::now();
        TaskNode {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Queued,
            priority: 50,
            dependencies: vec![],
            acceptance_criteria: vec![],
            agent: None,
            planned_action: None,
            retries: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Whether all dependencies are completed.
    pub fn dependencies_done(&self, graph: &TaskGraph) -> bool {
        self.dependencies
            .iter()
            .all(|id| matches!(graph.get(id).map(|t| t.status), Some(TaskStatus::Completed)))
    }
}

/// A directed graph of durable tasks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskGraph {
    /// The root task id.
    pub root: String,
    /// All tasks by id.
    pub tasks: BTreeMap<String, TaskNode>,
    /// The ordered, auditable transition history.
    pub log: Vec<Transition>,
    /// The last checkpoint (JSON snapshot), if any.
    pub checkpoint: Option<serde_json::Value>,
}

/// A single recorded transition for observability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    pub task_id: String,
    pub from: TaskStatus,
    pub to: TaskStatus,
    pub phase: String,
    pub note: String,
    pub at: DateTime<Utc>,
}

impl TaskGraph {
    /// Create a graph with a single root task.
    pub fn single(title: &str, description: &str) -> Self {
        let root = TaskNode::new("t-root", title, description);
        TaskGraph {
            root: root.id.clone(),
            tasks: BTreeMap::from([(root.id.clone(), root)]),
            log: vec![],
            checkpoint: None,
        }
    }

    /// Get a task by id.
    pub fn get(&self, id: &str) -> Option<&TaskNode> {
        self.tasks.get(id)
    }

    /// Get a mutable task by id.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut TaskNode> {
        self.tasks.get_mut(id)
    }

    /// Add a task to the graph.
    pub fn add(&mut self, task: TaskNode) {
        self.tasks.insert(task.id.clone(), task);
    }

    /// Record a transition for observability.
    pub fn record(
        &mut self,
        task_id: &str,
        from: TaskStatus,
        to: TaskStatus,
        phase: &str,
        note: &str,
    ) {
        self.log.push(Transition {
            task_id: task_id.to_string(),
            from,
            to,
            phase: phase.to_string(),
            note: note.to_string(),
            at: Utc::now(),
        });
    }

    /// The root task.
    pub fn root(&self) -> &TaskNode {
        self.tasks.get(&self.root).expect("root present")
    }

    /// Tasks in a given status.
    pub fn tasks_in(&self, status: TaskStatus) -> Vec<&TaskNode> {
        self.tasks.values().filter(|t| t.status == status).collect()
    }

    /// The first ready task whose dependencies are done (deterministic order).
    pub fn next_ready(&self) -> Option<&TaskNode> {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Ready)
            .filter(|t| t.dependencies_done(self))
            .min_by_key(|t| (std::cmp::Reverse(t.priority), t.id.clone()))
    }

    /// Whether every task is terminal (completed/failed/cancelled).
    pub fn is_terminal(&self) -> bool {
        self.tasks.values().all(|t| {
            matches!(
                t.status,
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
            )
        })
    }
}

/// The phases of the SDLC loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Phase {
    Plan,
    Decompose,
    Assign,
    Act,
    Verify,
    Repair,
    Checkpoint,
    Replan,
}

/// The Planner: turn a goal into a root plan.
///
/// Deterministic and injectable. In the real system this would consult a model;
/// here it is a pure function so the transition is testable. The planner marks
/// the root task `PLANNING` and records the plan.
pub struct Planner {
    /// A deterministic plan generator hook (e.g. a model or a fixed strategy).
    pub plan: PlanFn,
}

impl Default for Planner {
    fn default() -> Self {
        Planner {
            plan: Box::new(|title, description| format!("Plan for '{title}': {description}")),
        }
    }
}

impl Planner {
    /// Run the PLAN phase: record the plan on the root task.
    pub fn plan(&self, graph: &mut TaskGraph) -> String {
        let root_id = graph.root.clone();
        let (title, desc) = {
            let r = graph.root();
            (r.title.clone(), r.description.clone())
        };
        graph.record(
            &root_id,
            graph.root().status,
            TaskStatus::Planning,
            "PLAN",
            "planning",
        );
        if let Some(root) = graph.get_mut(&root_id) {
            root.status = TaskStatus::Planning;
            root.updated_at = Utc::now();
        }
        (self.plan)(&title, &desc)
    }
}

/// The Decomposer: break a task into sub-tasks with dependencies.
///
/// Deterministic and injectable. Returns the ids of created sub-tasks.
pub struct Decomposer {
    /// Given a task, produce sub-task (id, title, description) triples.
    pub decompose: DecomposeFn,
}

impl Decomposer {
    /// Run the DECOMPOSE phase, adding sub-tasks and moving the parent to READY.
    pub fn decompose(&self, graph: &mut TaskGraph, parent_id: &str) -> Vec<String> {
        let parent = graph.get(parent_id).cloned();
        let subs = match parent {
            Some(p) => (self.decompose)(&p),
            None => vec![],
        };
        let mut created = Vec::new();
        for (id, title, desc) in subs {
            let mut node = TaskNode::new(&id, &title, &desc);
            node.status = TaskStatus::Ready;
            node.acceptance_criteria = vec!["observed".to_string()];
            graph.add(node);
            created.push(id);
        }
        if let Some(p) = graph.get_mut(parent_id) {
            let from = p.status;
            graph.record(
                parent_id,
                from,
                TaskStatus::Ready,
                "DECOMPOSE",
                "decomposed",
            );
            let p = graph.get_mut(parent_id).unwrap();
            p.status = TaskStatus::Ready;
            p.updated_at = Utc::now();
        }
        created
    }
}

/// The Assigner: assign a ready task to an agent role.
pub struct Assigner {
    /// Choose an agent role for a task (e.g. based on capability fit).
    pub assign: AssignFn,
}

impl Assigner {
    /// Run the ASSIGN phase: set the agent and mark the task RUNNING.
    pub fn assign(&self, graph: &mut TaskGraph, task_id: &str) -> Option<String> {
        let agent = {
            let task = graph.get(task_id)?;
            if task.status != TaskStatus::Ready {
                return None;
            }
            (self.assign)(task)?
        };
        if let Some(t) = graph.get_mut(task_id) {
            let from = t.status;
            let note = format!("assigned {agent}");
            graph.record(task_id, from, TaskStatus::Running, "ASSIGN", &note);
            let t = graph.get_mut(task_id).unwrap();
            t.agent = Some(agent.clone());
            t.status = TaskStatus::Running;
            t.updated_at = Utc::now();
        }
        Some(agent)
    }
}
/// The TaskExecutor: run a task's action and return a result payload.
///
/// In the real system this runs the runtime's action through `execute()`; here
/// it is injectable so orchestration is testable without execution.
pub struct TaskExecutor {
    /// Run the action for a task, returning a result verdict + payload.
    pub run: RunFn,
}

/// The outcome of executing a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecResult {
    pub ok: bool,
    pub note: String,
    pub payload: serde_json::Value,
}

impl TaskExecutor {
    /// Run the ACT phase: mark RUNNING-and-execute, then move to VERIFYING.
    pub fn act(
        &self,
        graph: &mut TaskGraph,
        task_id: &str,
    ) -> Result<ExecResult, OrchestratorError> {
        let result = {
            let task = graph
                .get(task_id)
                .ok_or(OrchestratorError::UnknownTask(task_id.to_string()))?
                .clone();
            (self.run)(&task)
        };
        if let Some(t) = graph.get_mut(task_id) {
            t.status = TaskStatus::Verifying;
            t.updated_at = Utc::now();
        }
        graph.record(
            task_id,
            TaskStatus::Running,
            TaskStatus::Verifying,
            "ACT",
            &result.note,
        );
        Ok(result)
    }
}

/// A verification verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Pass,
    Fail,
}

/// The Verifier: check a result against acceptance criteria.
pub struct Verifier {
    /// Whether a task's result passes.
    pub check: CheckFn,
}

impl Verifier {
    /// Run the VERIFY phase: set COMPLETED on pass, else FAILED.
    pub fn verify(&self, graph: &mut TaskGraph, task_id: &str, result: &ExecResult) -> Verdict {
        let verdict = match graph.get(task_id) {
            Some(t) => (self.check)(t, result),
            None => Verdict::Fail,
        };
        let next = match verdict {
            Verdict::Pass => TaskStatus::Completed,
            Verdict::Fail => TaskStatus::Failed,
        };
        if let Some(t) = graph.get_mut(task_id) {
            let from = t.status;
            let note = if next == TaskStatus::Completed {
                "passed"
            } else {
                "failed"
            };
            graph.record(task_id, from, next, "VERIFY", note);
            let t = graph.get_mut(task_id).unwrap();
            t.status = next;
            t.updated_at = Utc::now();
        }
        verdict
    }
}

/// The Repairer: on failure, produce a repair plan (reopen for another attempt).
pub struct Repairer {
    /// Whether a failed task should be retried (turn count capped by the task).
    pub should_retry: RetryFn,
}

impl Repairer {
    /// Run the REPAIR phase: reopen a FAILED task to READY (repair) or leave it.
    pub fn repair(&self, graph: &mut TaskGraph, task_id: &str) -> bool {
        let retry = match graph.get(task_id) {
            Some(t) if t.status == TaskStatus::Failed => (self.should_retry)(t),
            _ => false,
        };
        if retry {
            if let Some(t) = graph.get_mut(task_id) {
                let from = t.status;
                graph.record(task_id, from, TaskStatus::Repairing, "REPAIR", "retrying");
                let t = graph.get_mut(task_id).unwrap();
                t.status = TaskStatus::Repairing;
                t.retries += 1;
                t.updated_at = Utc::now();
            }
            // Reopen to ready for another attempt.
            if let Some(t) = graph.get_mut(task_id) {
                let from = t.status;
                graph.record(task_id, from, TaskStatus::Ready, "REPAIR", "reopened");
                let t = graph.get_mut(task_id).unwrap();
                t.status = TaskStatus::Ready;
                t.updated_at = Utc::now();
            }
            true
        } else {
            false
        }
    }
}

/// The Checkpointer: snapshot the graph state for recovery.
pub struct Checkpointer;

impl Checkpointer {
    /// Run the CHECKPOINT phase: serialize the graph into a JSON snapshot.
    pub fn checkpoint(&self, graph: &mut TaskGraph) -> serde_json::Value {
        let snapshot = serde_json::to_value(&graph.tasks).expect("graph serializes");
        graph.checkpoint = Some(snapshot.clone());
        let root_id = graph.root.clone();
        let status = graph.root().status;
        graph.record(&root_id, status, status, "CHECKPOINT", "snapshotted");
        snapshot
    }
}

/// Errors produced by the orchestrator.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OrchestratorError {
    #[error("unknown task '{0}'")]
    UnknownTask(String),
    #[error("no ready task with satisfied dependencies")]
    NoReadyTask,
}

/// A plan generator: given a task title and description, produce a plan.
pub type PlanFn = Box<dyn Fn(&str, &str) -> String>;
/// A decomposer: given a task, produce sub-task (id, title, description)s.
pub type DecomposeFn = Box<dyn Fn(&TaskNode) -> Vec<(String, String, String)>>;
/// An assigner: choose an agent role for a task.
pub type AssignFn = Box<dyn Fn(&TaskNode) -> Option<String>>;
/// A task executor: run a task and produce an [`ExecResult`].
pub type RunFn = Box<dyn Fn(&TaskNode) -> ExecResult>;
/// A verifier: judge a result against a task.
pub type CheckFn = Box<dyn Fn(&TaskNode, &ExecResult) -> Verdict>;
/// A repairer decision: whether a failed task should be retried.
pub type RetryFn = Box<dyn Fn(&TaskNode) -> bool>;
/// The SDLC orchestrator: wires the phases into a single-step loop.
///
/// Each `advance()` call performs exactly one deterministic phase transition on
/// the [`TaskGraph`], so the loop is observable, testable, and recoverable.
pub struct Orchestrator {
    pub planner: Planner,
    pub decomposer: Decomposer,
    pub assigner: Assigner,
    pub executor: TaskExecutor,
    pub verifier: Verifier,
    pub repairer: Repairer,
    pub checkpointer: Checkpointer,
}

impl Orchestrator {
    /// Create an orchestrator wired with the given phase components.
    pub fn new(
        planner: Planner,
        decomposer: Decomposer,
        assigner: Assigner,
        executor: TaskExecutor,
        verifier: Verifier,
        repairer: Repairer,
    ) -> Self {
        Orchestrator {
            planner,
            decomposer,
            assigner,
            executor,
            verifier,
            repairer,
            checkpointer: Checkpointer,
        }
    }

    /// Run the PLAN phase.
    pub fn plan(&mut self, graph: &mut TaskGraph) -> String {
        self.planner.plan(graph)
    }

    /// Run the DECOMPOSE phase on the root.
    pub fn decompose(&mut self, graph: &mut TaskGraph) -> Vec<String> {
        self.decomposer.decompose(graph, &graph.root.clone())
    }

    /// Advance the loop by one transition.
    ///
    /// Deterministic scheduling: checkpoint, then assign/run the next ready
    /// task through ACT → VERIFY → (REPAIR). Returns the phase performed.
    pub fn advance(&mut self, graph: &mut TaskGraph) -> Phase {
        // Checkpoint first for recoverability.
        self.checkpointer.checkpoint(graph);

        // Pick the next ready task whose dependencies are done.
        let next = graph.next_ready().map(|t| t.id.clone());
        let task_id = match next {
            Some(id) => id,
            None => {
                // Nothing ready: if the root is still PLANNING, decompose it.
                if graph.root().status == TaskStatus::Planning {
                    self.decompose(graph);
                    return Phase::Decompose;
                }
                return Phase::Checkpoint;
            }
        };

        // Assign.
        if self.assigner.assign(graph, &task_id).is_none() {
            return Phase::Checkpoint;
        }

        // ACT.
        let result = match self.executor.act(graph, &task_id) {
            Ok(r) => r,
            Err(e) => {
                let _ = e;
                return Phase::Checkpoint;
            }
        };

        // VERIFY.
        let verdict = self.verifier.verify(graph, &task_id, &result);

        // REPAIR on failure.
        if verdict == Verdict::Fail {
            self.repairer.repair(graph, &task_id);
            return Phase::Repair;
        }

        Phase::Verify
    }

    /// Whether the loop is terminal (all tasks completed/failed/cancelled) and
    /// no task is ready or running.
    pub fn is_done(&self, graph: &TaskGraph) -> bool {
        graph.is_terminal() && graph.next_ready().is_none()
    }
}

/// A default verifier: a task passes when its execution result is ok.
pub fn default_verifier() -> Verifier {
    Verifier {
        check: Box::new(|_task, result| {
            if result.ok {
                Verdict::Pass
            } else {
                Verdict::Fail
            }
        }),
    }
}

/// A default repairer: retry a failed task up to 2 times.
pub fn default_repairer() -> Repairer {
    Repairer {
        should_retry: Box::new(|task| task.retries < 2),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Build an orchestrator whose executor always succeeds.
    fn ok_orchestrator() -> Orchestrator {
        Orchestrator::new(
            Planner::default(),
            Decomposer {
                decompose: Box::new(|t| {
                    vec![(
                        format!("{}-s1", t.id),
                        format!("{} sub 1", t.title),
                        "sub".to_string(),
                    )]
                }),
            },
            Assigner {
                assign: Box::new(|_| Some("developer".to_string())),
            },
            TaskExecutor {
                run: Box::new(|_| ExecResult {
                    ok: true,
                    note: "ok".to_string(),
                    payload: serde_json::json!({}),
                }),
            },
            default_verifier(),
            default_repairer(),
        )
    }

    #[test]
    fn plan_marks_root_planning() {
        let mut graph = TaskGraph::single("goal", "build a feature");
        let mut orch = ok_orchestrator();
        let plan = orch.plan(&mut graph);
        assert!(plan.contains("goal"));
        assert_eq!(graph.root().status, TaskStatus::Planning);
        assert!(!graph.log.is_empty());
    }

    #[test]
    fn decompose_adds_subtasks_with_dependencies() {
        let mut graph = TaskGraph::single("goal", "build a feature");
        let mut orch = ok_orchestrator();
        orch.plan(&mut graph);
        let created = orch.decompose(&mut graph);
        assert_eq!(created.len(), 1);
        // Parent is READY; subtask is READY and scheduled.
        assert_eq!(graph.root().status, TaskStatus::Ready);
        let sub = graph.get(&created[0]).unwrap();
        assert_eq!(sub.status, TaskStatus::Ready);
    }

    #[test]
    fn advance_runs_a_full_loop_to_completed() {
        let mut graph = TaskGraph::single("goal", "build a feature");
        let mut orch = ok_orchestrator();
        orch.plan(&mut graph);
        // First advance: decompose (parent planning -> ready + subtask).
        let phase = orch.advance(&mut graph);
        assert_eq!(phase, Phase::Decompose);
        // Subsequent advances drive root then subtask to completion.
        let mut steps = 0;
        while !orch.is_done(&graph) && steps < 10 {
            orch.advance(&mut graph);
            steps += 1;
        }
        assert!(orch.is_done(&graph));
        // Every non-container task reached a terminal state; log is populated.
        assert_eq!(graph.tasks_in(TaskStatus::Completed).len(), 2);
        assert!(!graph.log.is_empty());
        // A checkpoint snapshot was taken (recoverability).
        assert!(graph.checkpoint.is_some());
    }

    #[test]
    fn failed_task_is_repaired_and_retries() {
        let mut orch = ok_orchestrator();
        // Override executor: fail once, then succeed.
        let calls = std::cell::Cell::new(0u32);
        orch.executor = TaskExecutor {
            run: Box::new(move |_| {
                let n = calls.get();
                calls.set(n + 1);
                ExecResult {
                    ok: n >= 1,
                    note: "flaky".to_string(),
                    payload: serde_json::json!({}),
                }
            }),
        };
        let mut graph = TaskGraph::single("bad", "x");
        orch.plan(&mut graph);
        orch.decompose(&mut graph);
        // First attempt fails -> REPAIR retries (parent ready again).
        let phase = orch.advance(&mut graph);
        assert!(matches!(phase, Phase::Repair));
        // Second attempt succeeds -> COMPLETED.
        let phase2 = orch.advance(&mut graph);
        assert_eq!(phase2, Phase::Verify);
        assert_eq!(graph.tasks_in(TaskStatus::Completed).len(), 1);
    }

    #[test]
    fn checkpointer_snapshots_state() {
        let mut graph = TaskGraph::single("goal", "x");
        let cp = Checkpointer;
        let snapshot = cp.checkpoint(&mut graph);
        assert!(graph.checkpoint.is_some());
        assert!(snapshot["t-root"]["title"] == "goal");
    }

    #[test]
    fn next_ready_respects_dependencies() {
        let mut graph = TaskGraph::single("goal", "x");
        // A ready child with unmet dependency is not next_ready.
        let mut child = TaskNode::new("c1", "child", "c");
        child.status = TaskStatus::Ready;
        child.dependencies = vec!["t-root".to_string()];
        graph.add(child);
        // Root is not ready, child depends on root -> nothing ready.
        assert!(graph.next_ready().is_none());
    }
}
