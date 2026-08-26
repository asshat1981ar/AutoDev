# AutoDev Durable Development Harness v1.1

## Status
Approved reconciliation design for the AMCX-1 production release v1.1.

## Objective
Make AutoDev development portable, isolated, recoverable, evidence-driven, and safe across long-running or multi-session agentic work without changing AMX/ECM authority boundaries.

## Invariants
1. Existing Round 2 and Round 2.1 evidence is immutable; v1.1 supersedes packaging and harness behavior only.
2. Implementation work occurs in a task-scoped Git worktree under `.worktrees/<task-id>/` unless the execution environment provides a stronger native isolated workspace.
3. `.worktrees/` MUST be ignored by Git before a worktree is created.
4. The primary checkout is used for coordination and integration, not architectural implementation.
5. A durable checkpoint is written after every independently verified task and before any expected interruption.
6. Recovery trusts repository state, checkpoint state, verification receipts, and effect receipts over conversational recollection.
7. Skills and tools are procedural evidence, never authorization. Generated skills cannot self-activate or alter their evaluator.
8. Verification MUST execute against the unpacked release using repository-relative paths.
9. A release is not publishable unless clean-extraction verification passes.
10. Existing ForgeCore/ExecPlan/ECM/AMX/EvidenceStore authority boundaries remain unchanged.

## Mandatory process-skill batch
For code-changing development, the controller MUST invoke the applicable process skills in this order:
- design/brainstorming before behavioral or architectural changes;
- implementation planning for architectural or multi-session work;
- isolated-worktree setup before implementation;
- test-driven development for code changes;
- systematic debugging when observed behavior differs from expected behavior;
- verification-before-completion before claiming success;
- code review before integration.

Domain skills, connectors, and agents are selected dynamically using least-authority and smallest-sufficient-set rules. Mandatory process skills do not gain effect authority.

## Durable checkpoint contract
`.autodev/checkpoints/<task-id>.json` records:
- `schema_version`
- `task_id`
- `plan_path`
- `repository_revision`
- `branch`
- `worktree_path`
- `completed_tasks`
- `current_task`
- `verification`
- `effect_receipts`
- `rulings`
- `unresolved`
- `next_action`
- `updated_at`

Checkpoint writes are atomic (`.tmp` then replace). No secrets, tokens, raw credentials, or hidden reasoning may be stored.

## Agentic execution loop
`reconcile state -> inspect evidence -> select skills/tools -> isolate worktree -> plan -> TDD slice -> verify -> adversarial review -> correct -> reverify -> checkpoint -> integrate`

On recovery, the controller validates worktree/revision identity, reconciles external effects from receipts, reruns stale verification, and resumes only the first incomplete task.

## Release gate
The release builder MUST:
1. remove interpreter caches and transient files;
2. package repository-relative content;
3. extract the generated ZIP to a new temporary directory;
4. run `scripts/run_verification.sh` there;
5. emit a SHA-256 manifest only after the clean extraction passes.
