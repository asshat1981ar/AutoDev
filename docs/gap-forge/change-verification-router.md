# Change verification router candidate

Status: draft, disabled, advisory-only deterministic hook candidate.

## Classification

**Deterministic hook** — map successful file mutations to the narrowest repository verification command that should be run before push.

## Capability-gap evidence

AutoDev already states that every change must be checked with the narrowest relevant formatter, type checker, test, or build. Existing project-local support is not equivalent to this candidate:

- `.cline/hooks/pre_tool_use.py` is a destructive-command guard.
- `.cline/hooks/post_tool_use.py` emits telemetry only.
- `.cline/plugins/project-fabric/tools.py` exposes a generic `quality_gate_plan`, but it does not inspect changed paths or name concrete repository commands.
- `.cline/skills/release-readiness/SKILL.md` is a generic release checklist.
- `.cline/mcp/profiles.json` is for external systems; it does not provide repository-local change-aware verification routing.
- repository code search found no existing change-verification router.

The connected GitHub development surface can report CI results after a branch is pushed, but it does not provide edit-time local path-to-verifier context. This candidate therefore fills a distinct local deterministic-routing gap rather than duplicating an existing executor or external tool.

## Repeatable baseline

PR #33 (`fix/kotlin-sdk-licenses`) supplies the baseline failure. Its Android/Kotlin build path advanced beyond SDK installation, but a formatting follow-up still reached CI with seven ktlint violations in `kotlin/android-command-center/src/main/kotlin/dev/autodev/commandcenter/MainActivity.kt` (CI run `32157911839`).

This is a routing failure, not a missing verifier: the repository already has `ktlintCheck`; the missing capability is deterministic selection of that verifier from the path being changed.

The candidate fixture reproduces the relevant changed paths:

- `.github/workflows/ci.yml`
- `kotlin/android-command-center/src/main/kotlin/dev/autodev/commandcenter/MainActivity.kt`

Expected advisory commands are:

```text
cd kotlin && ./gradlew :android-command-center:ktlintCheck --no-daemon
python scripts/check_harness_drift.py
```

## RED -> GREEN evidence

### RED

Commit `5ed9e67ec4329024601f2f76f73d79568574e441` added tests before production code. CI run `32256093731` reached the existing Python suite and failed the new test module because `scripts.change_verification_router` did not exist. Existing tests completed successfully before that new import error.

### GREEN

Commit `c9ad7172251828de426f913e3b992a3fa3e8f390` added the smallest pure implementation. CI run `32256181683` confirms the complete Python 3.10 and Python 3.11 lanes are GREEN, including the full Cline development-fabric test suite and Termux launcher checks. The harness drift/reproducible lane is also GREEN.

The Kotlin lane on the same candidate run is not a candidate regression: it fails in unchanged `MainActivity.kt` on the existing deprecated `MediaType.parse(...)` call. PR #33 already changes that same line to the extension API and is the active repair branch for the Android/Kotlin CI surface. The verification-router candidate does not change any Kotlin, Gradle, workflow, ForgeCore, or Android files.

## Candidate behavior

`scripts/change_verification_router.py` is dependency-free and pure:

- successful `write_to_file`, `replace_in_file`, or `apply_patch` events may produce advisory context;
- Android command-center paths select the focused module `ktlintCheck`;
- other Kotlin paths select repository `ktlintCheck`;
- workflow paths select `scripts/check_harness_drift.py`;
- read-only or failed tools produce no context;
- command output is deterministically sorted;
- no suggested command mutates source (`ktlintFormat` is explicitly excluded by test).

The candidate is deliberately **not registered in active hook configuration** during evaluation.

## Trust and safety boundary

The candidate has no subprocess execution, shell invocation, filesystem writes, network access, ForgeCore calls, capability requests, credentials, approval handling, persistence, policy mutation, or public authority. `hook_response` always returns `cancel: false`; it can only inject advisory text naming a verification command.

ForgeCore remains the sole trusted execution authority. This candidate neither invokes nor bypasses it.

## Enablement gate

Keep the candidate disabled/unwired until a separate integration review confirms the Cline runtime hook registration contract and an enabled baseline-vs-candidate evaluation demonstrates that the advisory context reduces missed verification without introducing false blocking or safety regressions. Activation is out of scope for this PR.

## Rollback

Delete:

- `scripts/change_verification_router.py`
- `tests/test_change_verification_router.py`
- this document

No migration, credential rotation, policy rollback, approval reconciliation, data cleanup, or ForgeCore change is required.
