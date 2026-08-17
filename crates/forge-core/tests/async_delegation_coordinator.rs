use forge_core::async_delegation::{
    AsyncDelegationCoordinator, CoordinatorConfig, DelegatedOutput, DelegatedTask,
    DelegatedTaskExecutor, DelegationAssignment, DelegationClass, DelegationExecutionError,
    DelegationFuture, DelegationRisk,
};
use serde_json::json;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{Notify, Semaphore};

fn assignment() -> DelegationAssignment {
    DelegationAssignment::new("research-agent", ["repo:read"]).unwrap()
}

fn task(index: usize) -> DelegatedTask {
    DelegatedTask::new(
        format!("task-{index}"),
        "corr-1",
        None,
        assignment(),
        DelegationClass::ReadOnly,
        ["repo:read"],
        DelegationRisk::Low,
        "mock-provider",
        5_000,
        json!({"index": index}),
    )
    .unwrap()
}

#[derive(Clone)]
struct BlockingExecutor {
    current: Arc<AtomicUsize>,
    maximum: Arc<AtomicUsize>,
    started: Arc<AtomicUsize>,
    started_notify: Arc<Notify>,
    release_gate: Arc<Semaphore>,
}

impl BlockingExecutor {
    fn new() -> Self {
        Self {
            current: Arc::new(AtomicUsize::new(0)),
            maximum: Arc::new(AtomicUsize::new(0)),
            started: Arc::new(AtomicUsize::new(0)),
            started_notify: Arc::new(Notify::new()),
            release_gate: Arc::new(Semaphore::new(0)),
        }
    }
}

impl DelegatedTaskExecutor for BlockingExecutor {
    fn execute(&self, task: DelegatedTask) -> DelegationFuture {
        let current = Arc::clone(&self.current);
        let maximum = Arc::clone(&self.maximum);
        let started = Arc::clone(&self.started);
        let started_notify = Arc::clone(&self.started_notify);
        let release_gate = Arc::clone(&self.release_gate);

        Box::pin(async move {
            let in_flight = current.fetch_add(1, Ordering::SeqCst) + 1;
            maximum.fetch_max(in_flight, Ordering::SeqCst);
            started.fetch_add(1, Ordering::SeqCst);
            started_notify.notify_one();

            let permit = release_gate
                .acquire_owned()
                .await
                .map_err(|error| DelegationExecutionError::Executor(error.to_string()))?;
            drop(permit);
            current.fetch_sub(1, Ordering::SeqCst);

            Ok(DelegatedOutput {
                value: task.payload().clone(),
                evidence_fingerprint: None,
            })
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn coordinator_never_exceeds_global_concurrency_limit() {
    let executor = Arc::new(BlockingExecutor::new());
    let coordinator =
        AsyncDelegationCoordinator::new(CoordinatorConfig::new(3, 16).unwrap(), executor.clone());
    let tasks = (0..8).map(task).collect::<Vec<_>>();

    let handle = tokio::spawn(async move { coordinator.execute_batch(tasks).await.unwrap() });

    while executor.started.load(Ordering::SeqCst) < 3 {
        let notified = executor.started_notify.notified();
        if executor.started.load(Ordering::SeqCst) >= 3 {
            break;
        }
        notified.await;
    }

    tokio::task::yield_now().await;
    assert_eq!(executor.current.load(Ordering::SeqCst), 3);
    assert_eq!(executor.maximum.load(Ordering::SeqCst), 3);
    assert_eq!(executor.started.load(Ordering::SeqCst), 3);

    executor.release_gate.add_permits(8);
    let result = handle.await.unwrap();

    assert_eq!(result.results.len(), 8);
    assert_eq!(executor.maximum.load(Ordering::SeqCst), 3);
}

#[derive(Clone, Default)]
struct CountingExecutor {
    invocations: Arc<AtomicUsize>,
}

impl DelegatedTaskExecutor for CountingExecutor {
    fn execute(&self, task: DelegatedTask) -> DelegationFuture {
        let invocations = Arc::clone(&self.invocations);
        Box::pin(async move {
            invocations.fetch_add(1, Ordering::SeqCst);
            Ok(DelegatedOutput {
                value: task.payload().clone(),
                evidence_fingerprint: None,
            })
        })
    }
}

#[tokio::test]
async fn empty_batch_does_not_invoke_executor() {
    let executor = Arc::new(CountingExecutor::default());
    let coordinator =
        AsyncDelegationCoordinator::new(CoordinatorConfig::new(4, 16).unwrap(), executor.clone());

    let result = coordinator.execute_batch(vec![]).await.unwrap();

    assert!(result.results.is_empty());
    assert_eq!(executor.invocations.load(Ordering::SeqCst), 0);
}

#[test]
fn zero_global_concurrency_is_rejected() {
    assert!(CoordinatorConfig::new(0, 16).is_err());
}
