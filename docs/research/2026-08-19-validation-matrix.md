# ExecPlan validation matrix

| Invariant | Test/check |
| --- | --- |
| no serialized authority fields | `exec_plan_round_trips_without_authority_fields` |
| non-empty id/goal | `validation_rejects_invalid_identity_budget_and_milestones` |
| finite positive budgets | same validation test |
| unique milestone ids | same validation test |
| interrupted effect requires reconciliation | `interrupted_plan_requires_reconciliation_before_running` |
| bounded replanning | `replanning_is_bounded` |
| completion requires completed milestones | `completed_plan_requires_completed_milestones` |
| checkpoint preserves coordination state | `checkpoint_round_trip_preserves_coordination_state` |
| living PLANS.md sections exist | `tests.test_execplan_contract` + `scripts/check_execplan_contract.py` |
| compile/lint/format behavior | `.github/workflows/execplan.yml` |
