use forge_core::{ExecPlan, ExecPlanError, ExecPlanStatus, PlanBudget, PlanMilestone};

fn test_plan() -> ExecPlan {
    let mut plan = ExecPlan::new("plan-1", "Ship durable planning", PlanBudget::new(2, 3));
    plan.add_milestone(PlanMilestone::new("m1", "First milestone"))
        .unwrap();
    plan
}

#[test]
fn exec_plan_round_trips_without_authority_fields() {
    let plan = ExecPlan::new("plan-1", "Ship durable planning", PlanBudget::new(3, 5));
    let json = serde_json::to_string(&plan).unwrap();
    assert!(!json.contains("authorization_grant"));
    assert!(!json.contains("approved"));
    let restored: ExecPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.id(), "plan-1");
    assert_eq!(restored.status(), ExecPlanStatus::Planned);
}

#[test]
fn deserialization_rejects_completed_plan_with_incomplete_milestones() {
    let plan = test_plan();
    let mut persisted = serde_json::to_value(plan).unwrap();
    persisted["status"] = serde_json::Value::String("completed".into());

    let error = serde_json::from_value::<ExecPlan>(persisted).unwrap_err();
    assert!(error
        .to_string()
        .contains("all milestones must be complete before the plan can complete"));
}

#[test]
fn deserializes_the_legacy_exec_plan_wire_shape() {
    let persisted = serde_json::json!({
        "id": "legacy-plan",
        "goal": "Restore prior state",
        "status": "running",
        "references": {
            "task_ids": ["task-1"],
            "run_ids": ["run-1"],
            "envelope_ids": ["env-1"]
        },
        "budget": {
            "max_replans": 2,
            "max_attempts_per_milestone": 3,
            "replans_used": 1
        },
        "milestones": [{
            "id": "m1",
            "title": "Existing milestone",
            "completed": false,
            "attempts": 1
        }],
        "decisions": [],
        "discoveries": [],
        "interruption_reason": null,
        "created_at": "2026-08-18T12:00:00Z",
        "updated_at": "2026-08-18T12:30:00Z"
    });

    let restored: ExecPlan = serde_json::from_value(persisted.clone()).unwrap();

    assert_eq!(restored.id(), "legacy-plan");
    assert_eq!(serde_json::to_value(restored).unwrap(), persisted);
}

#[test]
fn validation_rejects_invalid_identity_budget_and_milestones() {
    let blank = ExecPlan::new(" ", "goal", PlanBudget::new(1, 1));
    assert_eq!(blank.validate(), Err(ExecPlanError::EmptyIdentityOrGoal));

    let zero_budget = ExecPlan::new("p", "g", PlanBudget::new(0, 1));
    assert_eq!(zero_budget.validate(), Err(ExecPlanError::InvalidBudget));

    let mut duplicate = test_plan();
    assert_eq!(
        duplicate.add_milestone(PlanMilestone::new("m1", "Duplicate")),
        Err(ExecPlanError::InvalidMilestones)
    );
}

#[test]
fn interrupted_plan_requires_reconciliation_before_running() {
    let mut plan = test_plan();
    plan.start().unwrap();
    plan.interrupt("process died during effect").unwrap();
    assert_eq!(
        plan.resume(false),
        Err(ExecPlanError::ReconciliationRequired)
    );
    plan.resume(true).unwrap();
    assert_eq!(plan.status(), ExecPlanStatus::Running);
}

#[test]
fn interrupted_plan_cannot_replan_around_reconciliation() {
    let mut plan = test_plan();
    plan.start().unwrap();
    plan.interrupt("effect outcome unknown").unwrap();

    assert_eq!(
        plan.consume_replan("try a different path"),
        Err(ExecPlanError::ReconciliationRequired)
    );
    assert_eq!(plan.status(), ExecPlanStatus::Interrupted);
    assert_eq!(plan.budget().replans_used(), 0);
}

#[test]
fn replanning_is_bounded() {
    let mut plan = ExecPlan::new("p", "g", PlanBudget::new(1, 2));
    plan.consume_replan("first").unwrap();
    assert_eq!(
        plan.consume_replan("second"),
        Err(ExecPlanError::ReplanBudgetExhausted)
    );
}

#[test]
fn completed_plan_requires_completed_milestones() {
    let mut plan = test_plan();
    plan.start().unwrap();
    assert_eq!(plan.complete(), Err(ExecPlanError::IncompleteMilestones));
    plan.complete_milestone("m1").unwrap();
    plan.complete().unwrap();
    assert_eq!(plan.status(), ExecPlanStatus::Completed);
    assert!(plan.validate().is_ok());
}

#[test]
fn checkpoint_round_trip_preserves_coordination_state() {
    let mut plan = test_plan();
    plan.add_task_reference("t-root");
    plan.add_run_reference("run-1");
    plan.add_envelope_reference("env-1");
    plan.record_decision("Keep plans authority-free", "ForgeCore owns authorization");
    plan.record_discovery("External effect reconciliation is required");
    plan.consume_replan("adjust milestone ordering").unwrap();

    let checkpoint = plan.checkpoint("cp-1").unwrap();
    let json = serde_json::to_string(&checkpoint).unwrap();
    let restored: forge_core::PlanCheckpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.plan_id(), plan.id());
    assert_eq!(restored.references(), plan.references());
    assert_eq!(restored.budget(), plan.budget());
    assert_eq!(restored.milestones(), plan.milestones());
    assert_eq!(restored.decisions(), plan.decisions());
    assert_eq!(restored.discoveries(), plan.discoveries());
}

#[test]
fn checkpoint_rejects_a_blank_identity_at_creation() {
    let plan = test_plan();

    assert_eq!(
        plan.checkpoint("   "),
        Err(ExecPlanError::InvalidCheckpointId)
    );
}

#[test]
fn milestone_attempts_are_bounded_and_persisted() {
    let mut plan = test_plan();
    plan.start().unwrap();

    assert_eq!(plan.start_milestone_attempt("m1").unwrap(), 1);
    assert_eq!(plan.start_milestone_attempt("m1").unwrap(), 2);
    assert_eq!(plan.start_milestone_attempt("m1").unwrap(), 3);
    assert_eq!(
        plan.start_milestone_attempt("m1"),
        Err(ExecPlanError::MilestoneAttemptBudgetExhausted)
    );

    let checkpoint = plan.checkpoint("cp-attempts").unwrap();
    assert_eq!(checkpoint.milestones()[0].attempts(), 3);
}

#[test]
fn validation_rejects_attempt_count_above_budget() {
    let plan = test_plan();
    let mut persisted = serde_json::to_value(plan).unwrap();
    persisted["milestones"][0]["attempts"] = serde_json::Value::from(4);

    let error = serde_json::from_value::<ExecPlan>(persisted).unwrap_err();
    assert!(error
        .to_string()
        .contains("milestone attempt budget exhausted"));
}

#[test]
fn milestone_attempt_rejects_unknown_milestone() {
    let mut plan = test_plan();
    plan.start().unwrap();
    assert_eq!(
        plan.start_milestone_attempt("missing"),
        Err(ExecPlanError::UnknownMilestone)
    );
}

#[test]
fn milestones_can_only_be_added_while_planned() {
    let mut plan = test_plan();
    plan.start().unwrap();

    assert_eq!(
        plan.add_milestone(PlanMilestone::new("m2", "Late milestone")),
        Err(ExecPlanError::InvalidTransition)
    );
}

#[test]
fn milestones_can_only_be_completed_while_active() {
    let mut plan = test_plan();

    assert_eq!(
        plan.complete_milestone("m1"),
        Err(ExecPlanError::InvalidTransition)
    );
}

#[test]
fn interrupted_work_cannot_bypass_reconciliation_through_blocking() {
    let mut plan = test_plan();
    plan.start().unwrap();
    plan.interrupt("effect result unknown").unwrap();

    assert_eq!(
        plan.block("hide interruption"),
        Err(ExecPlanError::InvalidTransition)
    );
    assert_eq!(
        plan.resume(false),
        Err(ExecPlanError::ReconciliationRequired)
    );
    assert_eq!(
        plan.consume_replan("retry without reconciling"),
        Err(ExecPlanError::ReconciliationRequired)
    );
}

#[test]
fn milestone_attempts_require_a_running_incomplete_milestone() {
    let mut plan = test_plan();
    assert_eq!(
        plan.start_milestone_attempt("m1"),
        Err(ExecPlanError::InvalidTransition)
    );

    plan.start().unwrap();
    plan.complete_milestone("m1").unwrap();
    assert_eq!(
        plan.start_milestone_attempt("m1"),
        Err(ExecPlanError::InvalidTransition)
    );
}

#[test]
fn add_milestone_rejects_deserialized_progress() {
    for persisted in [
        serde_json::json!({
            "id": "m2",
            "title": "Already completed",
            "completed": true,
            "attempts": 0
        }),
        serde_json::json!({
            "id": "m2",
            "title": "Already attempted",
            "completed": false,
            "attempts": 1
        }),
    ] {
        let milestone: PlanMilestone = serde_json::from_value(persisted).unwrap();
        let mut plan = test_plan();
        assert_eq!(
            plan.add_milestone(milestone),
            Err(ExecPlanError::InvalidMilestones)
        );
    }
}

#[test]
fn checkpoint_deserialization_rejects_invalid_coordination_state() {
    let plan = test_plan();
    let checkpoint = plan.checkpoint("cp-invalid").unwrap();
    let persisted = serde_json::to_value(checkpoint).unwrap();

    let mut incomplete_completed = persisted.clone();
    incomplete_completed["status"] = serde_json::Value::String("completed".into());
    assert!(serde_json::from_value::<forge_core::PlanCheckpoint>(incomplete_completed).is_err());

    let mut invalid_budget = persisted.clone();
    invalid_budget["budget"]["max_replans"] = serde_json::Value::from(0);
    assert!(serde_json::from_value::<forge_core::PlanCheckpoint>(invalid_budget).is_err());

    let mut excessive_attempts = persisted;
    excessive_attempts["milestones"][0]["attempts"] = serde_json::Value::from(4);
    assert!(serde_json::from_value::<forge_core::PlanCheckpoint>(excessive_attempts).is_err());
}

#[test]
fn terminal_plans_reject_replanning_without_consuming_budget() {
    let mut completed = test_plan();
    completed.start().unwrap();
    completed.complete_milestone("m1").unwrap();
    completed.complete().unwrap();

    assert_eq!(
        completed.consume_replan("mutate terminal plan"),
        Err(ExecPlanError::InvalidTransition)
    );
    assert_eq!(completed.budget().replans_used(), 0);

    let mut cancelled = test_plan();
    cancelled.cancel().unwrap();
    assert_eq!(
        cancelled.consume_replan("mutate cancelled plan"),
        Err(ExecPlanError::InvalidTransition)
    );
    assert_eq!(cancelled.budget().replans_used(), 0);
}
