use forge_core::{ExecPlan, ExecPlanError, ExecPlanStatus, PlanBudget, PlanMilestone};

fn test_plan() -> ExecPlan {
    let mut plan = ExecPlan::new("plan-1", "Ship durable planning", PlanBudget::new(2, 3));
    plan.milestones.push(PlanMilestone::new("m1", "First milestone"));
    plan
}

#[test]
fn exec_plan_round_trips_without_authority_fields() {
    let plan = ExecPlan::new("plan-1", "Ship durable planning", PlanBudget::new(3, 5));
    let json = serde_json::to_string(&plan).unwrap();
    assert!(!json.contains("authorization_grant"));
    assert!(!json.contains("approved"));
    let restored: ExecPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.id, "plan-1");
    assert_eq!(restored.status, ExecPlanStatus::Planned);
}

#[test]
fn validation_rejects_invalid_identity_budget_and_milestones() {
    let blank = ExecPlan::new(" ", "goal", PlanBudget::new(1, 1));
    assert_eq!(blank.validate(), Err(ExecPlanError::EmptyIdentityOrGoal));

    let zero_budget = ExecPlan::new("p", "g", PlanBudget::new(0, 1));
    assert_eq!(zero_budget.validate(), Err(ExecPlanError::InvalidBudget));

    let mut duplicate = test_plan();
    duplicate.milestones.push(PlanMilestone::new("m1", "Duplicate"));
    assert_eq!(duplicate.validate(), Err(ExecPlanError::InvalidMilestones));
}

#[test]
fn interrupted_plan_requires_reconciliation_before_running() {
    let mut plan = test_plan();
    plan.start().unwrap();
    plan.interrupt("process died during effect").unwrap();
    assert_eq!(plan.resume(false), Err(ExecPlanError::ReconciliationRequired));
    plan.resume(true).unwrap();
    assert_eq!(plan.status, ExecPlanStatus::Running);
}

#[test]
fn replanning_is_bounded() {
    let mut plan = ExecPlan::new("p", "g", PlanBudget::new(1, 2));
    plan.consume_replan("first").unwrap();
    assert_eq!(plan.consume_replan("second"), Err(ExecPlanError::ReplanBudgetExhausted));
}

#[test]
fn completed_plan_requires_completed_milestones() {
    let mut plan = test_plan();
    plan.start().unwrap();
    assert_eq!(plan.complete(), Err(ExecPlanError::IncompleteMilestones));
    plan.milestones[0].completed = true;
    plan.complete().unwrap();
    assert_eq!(plan.status, ExecPlanStatus::Completed);
    assert!(plan.validate().is_ok());
}

#[test]
fn checkpoint_round_trip_preserves_coordination_state() {
    let mut plan = test_plan();
    plan.references.task_ids.push("t-root".into());
    plan.references.run_ids.push("run-1".into());
    plan.references.envelope_ids.push("env-1".into());
    plan.record_decision("Keep plans authority-free", "ForgeCore owns authorization");
    plan.record_discovery("External effect reconciliation is required");
    plan.consume_replan("adjust milestone ordering").unwrap();

    let checkpoint = plan.checkpoint("cp-1").unwrap();
    let json = serde_json::to_string(&checkpoint).unwrap();
    let restored: forge_core::PlanCheckpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.plan_id, plan.id);
    assert_eq!(restored.references, plan.references);
    assert_eq!(restored.budget, plan.budget);
    assert_eq!(restored.milestones, plan.milestones);
    assert_eq!(restored.decisions, plan.decisions);
    assert_eq!(restored.discoveries, plan.discoveries);
}
