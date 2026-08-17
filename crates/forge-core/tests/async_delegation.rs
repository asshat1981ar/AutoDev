use forge_core::async_delegation::{
    validate_batch_for_v0, DelegatedTask, DelegationAssignment, DelegationClass, DelegationRisk,
    DelegationValidationError,
};
use serde_json::json;

fn assignment() -> DelegationAssignment {
    DelegationAssignment::new(
        "research-agent",
        ["repo:read", "model:invoke", "connector:read"],
    )
    .unwrap()
}

fn task(id: &str, class: DelegationClass) -> DelegatedTask {
    DelegatedTask::new(
        id,
        "corr-1",
        Some("parent-1".to_owned()),
        assignment(),
        class,
        ["repo:read"],
        DelegationRisk::Low,
        "mock-provider",
        1_000,
        json!({"path": "src/lib.rs", "question": "inspect"}),
    )
    .unwrap()
}

#[test]
fn v0_read_compute_and_observation_tasks_validate() {
    for class in [
        DelegationClass::ReadOnly,
        DelegationClass::Compute,
        DelegationClass::ExternalObservation,
    ] {
        assert_eq!(task("task-1", class).validate_for_v0(), Ok(()));
    }
}

#[test]
fn workspace_mutation_and_irreversible_tasks_fail_closed() {
    for class in [
        DelegationClass::WorkspaceMutation,
        DelegationClass::Irreversible,
    ] {
        assert_eq!(
            task("task-1", class).validate_for_v0(),
            Err(DelegationValidationError::UnsupportedClass(class))
        );
    }
}

#[test]
fn required_capability_must_be_inside_dispatch_assignment() {
    let error = DelegatedTask::new(
        "task-1",
        "corr-1",
        None,
        assignment(),
        DelegationClass::ReadOnly,
        ["repo:write"],
        DelegationRisk::Low,
        "mock-provider",
        1_000,
        json!({}),
    )
    .unwrap_err();

    assert_eq!(
        error,
        DelegationValidationError::CapabilityOutsideAssignment("repo:write".to_owned())
    );
}

#[test]
fn duplicate_task_ids_fail_batch_validation() {
    let first = task("task-1", DelegationClass::ReadOnly);
    let second = task("task-1", DelegationClass::Compute);

    assert_eq!(
        validate_batch_for_v0(&[first, second]),
        Err(DelegationValidationError::DuplicateTaskId(
            "task-1".to_owned()
        ))
    );
}

#[test]
fn identical_inputs_have_stable_fingerprints() {
    let first = task("task-1", DelegationClass::ReadOnly);
    let second = task("task-1", DelegationClass::ReadOnly);
    assert_eq!(first.input_fingerprint(), second.input_fingerprint());
}

#[test]
fn security_relevant_payload_change_changes_fingerprint() {
    let first = task("task-1", DelegationClass::ReadOnly);
    let second = DelegatedTask::new(
        "task-1",
        "corr-1",
        Some("parent-1".to_owned()),
        assignment(),
        DelegationClass::ReadOnly,
        ["repo:read"],
        DelegationRisk::Low,
        "mock-provider",
        1_000,
        json!({"path": "src/runtime.rs", "question": "inspect"}),
    )
    .unwrap();

    assert_ne!(first.input_fingerprint(), second.input_fingerprint());
}

#[test]
fn empty_batch_is_valid() {
    assert_eq!(validate_batch_for_v0(&[]), Ok(()));
}
