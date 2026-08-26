# AutoDev Self-Evaluation Factory

AutoDev's self-evaluation factory is a repository-local experiment harness for comparing development configurations against a fixed set of executable historical tasks.

It answers one bounded question:

> Did the candidate configuration solve strictly more comparable tasks without introducing a safety regression or weakening the verifier?

Evaluation evidence is **not execution authority**. It does not mint `AuthorizationGrant` values, widen capabilities, change policy, merge code, activate skills, install MCPs, or promote a candidate automatically.

## Trust boundary

```text
forge-core::evaluation
  task identity + validation
  deterministic fingerprints
  outcome derivation
  report aggregation
  baseline/candidate comparison
          ↑ normalized data only
          │
autodev-eval
  curated fixture loading
  isolated historical checkout
  injected AttemptDriver
  changed-path capture
  hidden verifier overlay
  structured verifier execution
  evidence normalization
```

`forge_core::evaluation` is side-effect free. Git materialization and verifier subprocesses live in the `autodev-eval` adapter outside ForgeCore's trusted authorization boundary.

The evaluated development path is injected through `AttemptDriver`; v0 does not add an unrestricted host-shell development driver.

## Outcome semantics

A task is `solved` only when every required verifier step has one execution-backed passing evidence record and no integrity/safety finding invalidates the attempt.

A verifier assertion is execution-backed only when the verifier actually started, did not time out, exited with code 0, and produced normalized stdout/stderr evidence digests. Agent text such as “tests pass” has no scoring effect.

`infrastructure_failure` is separate from `unsolved`. A missing host verifier executable, unavailable historical revision, corrupted hidden verifier asset, or evaluator failure is excluded from the verified-success denominator rather than counted against the development configuration.

Safety findings are an independent axis. Any candidate safety regression prevents an `improved` comparison result even if its functional solve rate is higher.

## Hidden verifier model

Fixture definitions and hidden verifier assets remain outside the evaluated checkout and are not supplied as answer context to the attempt driver.

For a task that needs a hidden probe, the runner performs this sequence:

1. materialize the pinned base revision;
2. run the development attempt;
3. capture the attempt's changed paths;
4. flag any attempt-created hidden verifier destination as `verifier_overlay_collision`;
5. verify each hidden asset SHA-256 digest;
6. copy the verifier-owned bytes into the temporary checkout;
7. execute the fixed structured verifier recipe.

Hidden asset digests are part of the verifier fingerprint. Changing a hidden probe therefore changes verifier identity and makes old/new reports incomparable.

Verifier programs use structured `program + args + working_directory + timeout_seconds`; opaque shell command strings are rejected. Stdout and stderr are drained concurrently to avoid pipe deadlock, with bounded evidence retained for hashing.

## Historical corpus v0

| Task | Base SHA | Accepted/reference SHA | Primary verifier |
| --- | --- | --- | --- |
| `architecture-evidence-forge` | `5c0adf94d192aef131c96d4cb72ef00e30bf7501` | `6df35bf674af8023779f59b6770135dca2895d74` | hidden Rust integration probe |
| `termux-kanban-pty-repair` | `8ee7ccc72e7a342e9029c90d7ed311ae11a3ec9b` | `4e0c35551c890cae71ab7b9af843dac86eaa3d78` | Node syntax + launcher check mode |
| `android-command-center` | `16a13c492e3992e0288f1638d46d79d021cc2fc2` | `c4b13fb66b427ea6b7c6c57f823962b4def818b7` | Gradle debug APK assembly |
| `rust-control-plane-secure-webhook` | `c4b13fb66b427ea6b7c6c57f823962b4def818b7` | `5c0adf94d192aef131c96d4cb72ef00e30bf7501` | hidden Axum/Tower integration probe |
| `kmp-rebuild-toolchain` | `4227749db45624e539ab159c09bc804a5d815fa8` | `85f0c2ba2c58e5e4183a210d3ebf6c4509b451dc` | Gradle assemble/test/ktlint |

The KMP task deliberately invokes the repository wrapper through the structured host `env` executable. Its base revision predates `kotlin/gradlew`; this makes the missing wrapper an evaluated task-state failure rather than incorrectly classifying it as a missing host verifier executable.

## CLI

From `crates/`:

```bash
cargo run -p autodev-eval -- validate --fixtures autodev-eval/fixtures
cargo run -p autodev-eval -- smoke --fixtures autodev-eval/fixtures --source-repo ..
cargo run -p autodev-eval -- compare --baseline baseline.json --candidate candidate.json
```

`validate` loads all curated JSON fixtures and prints deterministic task and verifier fingerprints.

`smoke` requires a source checkout containing full history. It succeeds only when every task's base state fails its fixed verifier and every accepted/reference state passes the same verifier. **An empty corpus fails the smoke gate** — the runner refuses to report healthy on `results == []` because a hostile or accidental clear of `fixtures/` must not silently pass.

`compare` prints a typed comparison. The exit code reflects only whether the comparison could be computed; it does **not** signal that the candidate was `improved`. A successful exit with a `no_improvement` decision still means "the candidate did not improve on the baseline". A CI gate that wants to enforce improvement must inspect the printed `decision` field, not the exit code.

### Fixture-asset integrity

Hidden verifier assets live under `crates/autodev-eval/fixture-assets/`. Every
`verifier_overlay` entry in a fixture declares the SHA-256 of the asset it
overlays, and `apply_verifier_overlays` re-hashes the asset on disk before
copying it into the runner checkout. **A drift between the asset on disk and
the digest declared in the fixture fails the smoke gate with an integrity
error.** This makes "swap the probe to make the base state pass" structurally
impossible without also editing the fixture and the asset, and editing the
fixture to match is a visible PR change.

The integrity model assumes **PR-reviewer approval** is the human gate for
new or modified fixture assets. Any change to `fixture-assets/**` is by
definition a change to the verifier identity; reviewers must treat
modifications to those files with the same scrutiny as a change to a unit
test fixture. Verifier steps are *host code execution* and should never be
treated as advisory.

## Comparison rules

Reports are comparable only when their complete ordered task keys match. Each task key includes both the task fingerprint and verifier fingerprint.

For comparable reports:

- `improved` requires a strictly higher verified success rate and zero candidate safety regressions;
- `safety_regression` wins over a functional improvement when the candidate has any safety finding;
- equal or lower verified success is `no_improvement`;
- task or verifier drift is `incomparable`.

There is no automatic promotion step in v0.

## Adding a curated task

A new task must have:

1. a stable lowercase task slug;
2. an immutable full base SHA;
3. an immutable accepted/reference SHA;
4. source provenance;
5. a specification and acceptance criteria;
6. at least one fixed required verifier step;
7. structured verifier argv and a bounded timeout;
8. explicit expected-change and protected surfaces;
9. content-addressed hidden verifier assets when used;
10. evidence that the base fails and the accepted/reference state passes the same verifier.

Do not expose the historical implementation patch as an answer oracle to the evaluated attempt.

## Verification

Rust/evaluator gates:

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
cargo run -p autodev-eval -- validate --fixtures autodev-eval/fixtures
```

Full historical smoke requires repository history plus Java/Android/Node toolchains:

```bash
cd crates
AUTODEV_EVAL_SOURCE_REPO="$(git rev-parse --show-toplevel)" \
  cargo test -p autodev-eval --test corpus_smoke -- --ignored --nocapture
```

A benchmark infrastructure failure is a failing evaluation gate. It must not be silently converted into a solved or unsolved development attempt.
