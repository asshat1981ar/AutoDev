# Work Chat ↔ Drive Reconciliation v1.1

## Baseline
Source release: `AutoDev-AMCX1-Complete-Production-Release.zip`.
The Round 2 report and Round 2.1 correction/supersession remain preserved; this reconciliation does not rewrite their semantic adjudication.

## Retained
- AMX owns portable memory history.
- ECM owns collaboration history and promotion evidence, not effect authority.
- ForgeCore/ExecPlan/EvidenceStore authority boundaries remain external and unchanged.
- Gate PASS is evidence, never activation authority.
- Generated skills cannot self-promote.

## Corrected
- Removed build-host-specific `/working_dir/...` assumptions from tests.
- Verification now establishes `src` relative to the extracted repository.
- Verification success totals are discovered at runtime rather than asserted by a static message.

## Added from later work-chat direction
- Mandatory lifecycle process-skill batch.
- Task-scoped `.worktrees/<task-id>/` implementation isolation.
- Durable `.autodev/checkpoints/<task-id>.json` recovery state.
- Clean-extraction release gate.
- Versioned Mistral Vibe v1.1 execution instructions.
- Release builder with transient-file exclusion and detached SHA-256 output.

## Verification protocol
1. Run the complete unit suite through `scripts/run_verification.sh`.
2. Build v1.1 with `scripts/build_release.py`.
3. Extract the generated ZIP into a second independent temporary directory.
4. Run `scripts/run_verification.sh` again from that extraction.
5. Treat the detached SHA-256 as release identity only after step 4 succeeds.
