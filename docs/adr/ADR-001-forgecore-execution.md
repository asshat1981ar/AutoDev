# ADR-001: ForgeCore Trusted Execution Kernel

- **Status:** Accepted (foundation)
- **Date:** 2026-08-09
- **Deciders:** Principal systems architect
- **Related:** `docs/architecture/repository-assessment.md`, `docs/architecture/agent-protocol.md`, `protocols/*.schema.json`
- **Scope:** The execution boundary of the AutoDev platform. This ADR selects the
  strategy for *securely executing* authorized agent intent inside the Rust ForgeCore
  kernel. It does **not** define the orchestrator, model fabric, or control plane.

---

## 1. Context

AutoDev's core promise is **safe autonomous software development**: agents propose
*intent*, a policy layer *authorizes* it, and a trusted kernel *executes* only the
authorized operations while producing verifiable evidence.

The architectural boundary this ADR governs is:

```text
Agent
  → AgentAction          (typed intent)
  → Validation           (structural invariants)
  → Capability           (declared capability set)
  → Policy               (risk/resource authorization)
  → Approval             (human-in-the-loop for high risk)
  → Execution            (ForgeCore adapters, sandboxed)
  → Evidence             (schema-conformant ExecutionResult)
```

Today, `forge-core` implements only the *left* side of this boundary: `ActionType`,
`RiskLevel`, `AgentAction`, `PolicyDecision`, `PolicyError`, `validate_action`,
`evaluate_policy`, and a `dry_run` executor that deliberately performs **no** privileged
effects. The `ExecutionResult` type is a stub. There is **no** real execution adapter,
no sandbox, and no process/fs/network access.

The decision recorded here is the strategy for the **Execution** step and its
**sandbox** — the highest-risk, highest-value missing capability identified in the
repository assessment.

### 1.1 Requirements and constraints

| Constraint | Implication |
| --- | --- |
| **Local-first & offline** | Sandbox must not depend on a network registry or cloud service at runtime. |
| **Multi-OS** | Android (primary target), Linux, Windows, macOS all in the roadmap. |
| **Performance** | Execution must be fast enough for interactive agent loops; low overhead per action. |
| **Portability** | A single execution abstraction should degrade gracefully across OSes. |
| **Security-critical** | The kernel is the trust boundary between untrusted model output and the host. |
| **Typed intent** | No arbitrary shell text; actions are typed and validated. |
| **Evidence** | Every execution must return a schema-conformant, durable `ExecutionResult`. |

---

## 2. Decision drivers

1. **Trust separation** — model output (agent reasoning) is *never* trusted; the kernel
   is the enforcement point.
2. **Least privilege** — execution runs with the minimum rights needed, against a
   confined workspace.
3. **Offline operability** — no hard dependency on container daemons, network, or
   cloud sandboxes.
4. **Portability-first** — the mechanism must work (with varying strength) on
   Android, Linux, Windows, and macOS.
5. **Incrementality** — the first implementation must be small enough to validate
   quickly and to harden before the Kotlin/orchestrator layers are built.
6. **Determinism & evidence** — executions produce structured, replayable evidence,
## 3. Considered approaches

### 3.1 Workspace / path sandboxing

**Description.** Confine every operation to a designated workspace root by
canonicalizing and validating all paths against an allow-list of roots (and optional
deny-list). Reads/writes are mediated by the kernel in-process; symlinks are resolved
and ancestry is checked before any operation.

**Strengths.** Simple, fast (no context switch), fully portable, works offline, no
external runtime. Maps directly onto the typed `AgentAction` model (paths are payload
fields). Easy to test.

**Weaknesses.** Only protects the *filesystem*; does not confine processes, network, or
system calls. A process-execution adapter (see roadmap) is not contained by path checks
alone. Path semantics differ across OSes; symlink/hardlink and `..` escapes must be
handled carefully. Defense is in the application layer, not the OS.

### 3.2 OS-level process isolation

**Description.** Run each execution (or each agent "session") as an OS process with
dropped privileges and OS-enforced limits:

- **Linux:** `setrlimit`, seccomp-bpf filters, and **Landlock** (unprivileged LSM) to
  confine filesystem/network access to specific hierarchies and ports; optionally
  namespaces.
- **macOS:** `sandbox-exec` / `sandbox_init(3)` with `.sb` profiles; hardened runtime;
  App Sandbox for sandboxed apps.
- **Windows:** Job Objects + restricted tokens + AppContainer / Integrity Levels.
- **Android:** SEAndroid / per-app sandbox + `isolatedProcess`; the app is already
  sandboxed by the platform.

**Strengths.** Real OS enforcement of filesystem, network, and syscall limits; works
offline; strong security when correctly configured; leverages platform-native,
well-audited mechanisms.

**Weaknesses.** Strongest primitives are OS-specific → a portability abstraction is
required. Configuring seccomp/Landlock/sandbox profiles correctly is complex and
version-sensitive (e.g., Landlock ABI differences). Process spawning has overhead vs.
in-process calls. Sandbox strength varies by platform (Android is strongest by default;
Windows/macOS require more setup).

### 3.3 WASM capability execution

**Description.** Compile/execute agent-supplied logic as WebAssembly modules governed by
**WASI** (capability-based). WASI grants access only to explicitly passed file
descriptors/directories, environment, and clocks. Runtimes (e.g., Wasmtime, Wasmer)
provide capability-sandboxed filesystem, network, and clock access by default.

**Strengths.** Fine-grained capability model; deterministic; memory-safe; portable
across OSes; strong isolation of the *logic being executed*; great for running
untrusted agent "tools" or probe code. No root required.

**Weaknesses.** Only sandboxes WASM code, not arbitrary native programs. Running a
user's real toolchain (compilers, test runners, git) as WASM is immature/impossible for
most tools. FFI/guest↔host boundary adds complexity and latency. Not a drop-in for
executing existing native binaries. WASI is still evolving (preview versions).

### 3.4 Containerized execution

**Description.** Run each action/session in an OCI container (Docker/Podman) or
lightweight VM (Firecracker, gVisor) with a read-only rootfs, mounted workspace, and
dropped capabilities.

**Strengths.** Strong, well-understood isolation; reproducible environments; good for
CI-like workloads; network/fs policy via container config.

**Weaknesses.** **Poor offline/serverless fit on Android** (Android cannot run Docker
natively). Heavy runtime requirement (daemon, image management). High startup latency
and resource overhead for per-action execution. Image supply chain and updates add
complexity. Not portable to all roadmap targets. Overkill for granular typed actions.

### 3.5 Hybrid approaches

**Description.** Compose layers: **policy-native workspace confinement** as the
always-on, portable base (defense in depth + evidence), plus **OS-level process
isolation** as the second tier for anything that spawns processes or touches the
network, plus an **optional WASM capability tier** for executing untrusted agent code.
Containers are reserved for a later, optional "heavy build worker" path.

**Strengths.** Addresses each threat at the right layer; degrades gracefully across
OSes (Android gets policy + platform sandbox; Linux adds Landlock + seccomp; macOS adds
sandbox profiles; Windows adds restricted tokens); offline-friendly; scalable from MVP
to production.

**Weaknesses.** More moving parts than a single mechanism; requires a clear layering
contract and consistent policy model across tiers; more upfront design discipline.

### 3.6 Comparison scorecard

Scored 1–5 (5 = best). "Complexity" is inverted so that 5 = simplest; "Security" weighs
the strongest achievable guarantee and the portability of that guarantee.

| Approach | Security | Complexity | Maintainability | Performance | Portability | Offline | Extensibility | Testability | **Total** |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 3.1 Path sandbox | 2 | 5 | 5 | 5 | 5 | 5 | 3 | 5 | **35** |
| 3.2 OS isolation | 4 | 3 | 3 | 4 | 3 | 5 | 4 | 3 | **29** |
| 3.3 WASM/WASI | 4 | 3 | 3 | 3 | 5 | 5 | 4 | 4 | **31** |
| 3.4 Containers | 5 | 1 | 2 | 2 | 1 | 3 | 3 | 2 | **19** |
| 3.5 Hybrid | 5 | 3 | 4 | 4 | 4 | 5 | 5 | 4 | **34** |

The hybrid approach scores highest on the overall portfolio: it delivers the strong
security of OS isolation while keeping the portable, offline, and testable base of path
confinement, and it reserves WASM as a future extension point rather than betting the
foundation on a single mechanism.
## 4. Decision

**Adopt a layered hybrid execution kernel** with three tiers, in decreasing order of
dependence:

1. **Workspace confinement (mandatory, always-on, portable).** ForgeCore hosts a
   *Workspace* abstraction that canonicalizes and validates every path against
   configured allow-listed roots before any operation. This is the foundation and the
   cross-platform guarantee. It is the only tier required for the MVP.
2. **OS-level process isolation (mandatory for process/network execution; capability-
   graded per OS).** When an authorized action spawns a process or touches the network,
   it runs under the platform's native confinement: Linux **Landlock + seccomp** (with
   `setrlimit`), macOS **sandbox profile** (`sandbox_init`), Windows **restricted token
   + Job Object / AppContainer**, Android **platform app sandbox / `isolatedProcess`**.
   A thin *SandboxAdaptor* trait abstracts these so the policy layer is platform-
   neutral.
3. **WASM capability execution (optional, future).** For executing *untrusted agent
   code / tools*, add a WASI-based capability sandbox (Wasmtime/Wasmer) that grants
   only explicitly passed directories and capabilities. This is a deliberate future
   extension, not required for the MVP.

Containers/lightweight VMs are **explicitly out of scope** for the core kernel and are
reserved as an optional, later "heavy build worker" deployment path.

### 4.1 Why this decision

- It makes the **portable, offline, testable guarantee** (tier 1) the default, so even
  the MVP on Android is meaningfully safe.
- It layers **real OS enforcement** (tier 2) exactly where the threat is highest
  (process/network), which no amount of path-checking alone can provide.
- It leaves **WASM** (tier 3) as an opt-in capability boundary rather than forcing the
  hard, immature problem of running arbitrary native toolchains as WASM.
- It keeps the **dependencies minimal** today (no Docker, no WASM runtime in the core),
  consistent with the assessment's finding that the current dependency surface is
  appropriately small.

---

## 5. Responsibilities

ForgeCore's execution kernel is responsible for:

- **Authorizing intent** — enforcing the `Validation → Capability → Policy → Approval`
  pipeline so that only authorized `AgentAction`s reach execution.
- **Confining operations** — restricting every effect to the caller's workspace root
  (tier 1) and dropping privileges for process/network work (tier 2).
- **Executing typed adapters** — `read_file`, `write_file`, `patch_file`, `execute`,
  `git`, `run_test`, and (future) `mcp` adapters, each taking a validated action and
  returning a result.
- **Producing evidence** — returning a schema-conformant `ExecutionResult` (status,
  exit code, stdout/stderr, artifacts, verification, timestamps).
- **Recording provenance** — attaching action identity, task/agent identity, policy
  decision, and timestamps to every result.
- **Path safety** — canonicalizing, normalizing, and validating all paths against the
  workspace allow-list; rejecting symlink/`..` escapes.
- **Resource limits** — enforcing timeouts, memory, and output-size limits per action.
- **Approval gating** — surfacing `RequireApproval` decisions to the orchestrator and
  refusing execution until approval is recorded.

## 6. Non-responsibilities

ForgeCore's execution kernel is **not** responsible for:

- **Agent reasoning, planning, or model selection** — that is the orchestrator/model
  fabric's job.
- **Orchestration state machines** (PLAN/ACT/VERIFY/REPLAN) — handled by the Kotlin
  control plane.
- **UI / human approval presentation** — the kernel only records the need for approval;
  it does not render it.
- **Long-term durable storage of tasks/agents** — handled by the persistence layer.
- **Image/container management** — explicitly deferred.
- **Network-strong isolates** — the kernel does not provide its own networking; it
  relies on OS sandbox primitives when network execution is authorized.
- **Guaranteeing isolation of arbitrary native binaries** — tier 1 confines paths only;
  tier 2 confines processes; neither is a full VM. The kernel does not claim
  virtualization-grade isolation.
## 7. Trust boundaries

```text
             UNTRUSTED                          TRUSTED
 ┌──────────────────────────┐        ┌──────────────────────────────────┐
 │  Agent / model output    │        │  Orchestrator (control plane)    │
 │  (never trusted)         │        │  (trusted by policy, untrusted   │
 └───────────┬──────────────┘        │   as to capability)              │
             │ AgentAction           └───────────────┬──────────────────┘
             ▼                                       │
 ┌──────────────────────────┐                        │
 │  Protocol / schema layer │                        │
 │  (validates shape)       │                        │
 └───────────┬──────────────┘                        │
             ▼                                       │
 ┌────────────────────────────────────────────────────┴──────────────┐
 │                ForgeCore KERNEL (trusted)                          │
 │  Validation → Capability → Policy → Approval                       │
 │  ┌──────────────────┐   ┌──────────────────┐   ┌─────────────────┐ │
 │  │ Workspace (tier1)│   │ SandboxAdaptor   │   │ WASM (tier3)    │ │
 │  │ path confinement │   │ (tier2, per-OS)  │   │ capability sand.│ │
 │  └────────┬─────────┘   └────────┬─────────┘   └────────┬────────┘ │
 └───────────┼──────────────────────┼──────────────────────┼──────────┘
             ▼                      ▼                      ▼
        Host filesystem        OS sandbox            WASI capabilities
        (workspace roots)      (Landlock/seccomp,    (explicit dirs,
                               sandbox, token)        clocks, env)
```

**Boundary rules:**

1. **Agent/model output is untrusted** at all times. It becomes actionable only through
   typed `AgentAction` that has passed validation, capability, policy, and (if required)
   approval.
2. **The kernel is the single trust anchor.** Policy evaluation, confinement, and
   evidence generation all execute inside ForgeCore, which is trusted.
3. **No privileged operation occurs without policy authorization AND confinement.**
   `dry_run`'s invariant ("refuses privileged effects") is preserved: real adapters only
   run inside a confined workspace.
4. **The orchestrator is trusted for policy input but not for capability.** It requests;
   the kernel authorizes. The orchestrator cannot grant itself more rights than policy
   allows.
5. **The OS is trusted** (platform runtime, syscall enforcement). The kernel relies on
   OS primitives for tier-2 isolation and does not re-implement them.
6. **Approval is a gate, not a capability.** Approval records unblock execution but do
   not expand the workspace or capability set.

---

## 8. Interfaces

ForgeCore's public API (`lib.rs`) evolves to expose a clean, versioned execution
interface. Core types (all `serde`-serializable, language-neutral):

```rust
/// A configured, allow-listed workspace root.
pub struct Workspace {
    root: PathBuf,
    allowed_roots: Vec<PathBuf>,
    max_bytes: u64,
    max_timeout: Duration,
}

/// Result of resolving a payload path against the workspace.
pub enum PathResolution {
    Allowed(PathBuf),   // canonical path inside an allow-list root
    Denied(PathBuf),    // canonical path outside allowed roots
    Invalid(String),    // malformed / non-UTF8 / escape attempt
}

/// Platform-neutral confinement for process/network execution.
pub trait SandboxAdaptor {
    fn spawn(&self, spec: &SandboxSpec) -> io::Result<Child>;
    // … resource limits, termination, wait …
}

/// A single validated, authorized action ready for execution.
pub struct ExecutableAction {
    pub action: AgentAction,     // already passed policy + approval
    pub workspace: Workspace,
    pub capabilities: Vec<String>,
}

/// The canonical, schema-conformant execution evidence.
pub struct ExecutionResult {
    pub action_id: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub artifacts: Vec<String>,
    pub verification: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Entry point: only authorized + operator-approved actions are executed.
pub fn execute(exec: ExecutableAction) -> Result<ExecutionResult, ExecutionError>;
```

**Interface rules:**

- `execute` is the **only** entry that can produce side effects, and it requires an
  `ExecutableAction` that has already passed the full gate. `dry_run` remains for
  preview and is the same code path with confinement but no committed effects.
- All adapters (read/write/patch/execute/git/run_test) implement a common `ActionAdapter`
  trait returning `ExecutionResult`, so policy and evidence handling are uniform.
- Persistence and serialization use the existing `protocols/*.schema.json` contracts;
  `ExecutionResult` is aligned to `execution-result.schema.json` (fixing the drift noted
  in the repository assessment).
- RPC/FFI: ForgeCore is consumed by Kotlin later via JNI/FFI; all boundary types are
  `serde`-serializable JSON so the same contract works across Kotlin, Rust, and future
  Go workers.
## 9. Threat model

**Assumptions / trust boundaries** (from §7): agent/model output untrusted; the kernel
trusted; the OS trusted; the orchestrator trusted for policy input but not for
capability.

| # | Threat | Target | Mitigation (tier) |
| --- | --- | --- | --- |
| T1 | **Path traversal / escape** — `../../`, absolute paths, symlink/hardlink out of workspace | Host filesystem | Canonicalize + allow-list ancestry check; reject non-canonical paths; symlink resolution (tier 1) |
| T2 | **Malicious file write** — overwrite host/sensitive files | Host filesystem | Workspace confinement + read/write allow-list (tier 1); denied-by-default policy |
| T3 | **Process execution abuse** — action spawns arbitrary/unsafe process | Host | Allow-listed executables + OS isolation (tier 2); resource limits; seccomp/sandbox |
| T4 | **Network exfiltration / unsafe network** | Host/LAN | Landlock/sandbox network rules; deny-by-default; no network in MVP (tier 2) |
| T5 | **Syscall abuse via untrusted code** | Host kernel | Tier-2 syscall filtering (seccomp) / sandbox profile; optionally tier-3 WASM |
| T6 | **Resource exhaustion** — infinite loop, huge output, memory blowup | Host availability | Timeouts, output byte caps, memory limits (`setrlimit`/Job)/WASM fuel (tiers 2/3) |
| T7 | **Policy bypass** — action reaches executor without authorization | Kernel integrity | `execute` only accepts pre-gated `ExecutableAction`; single entry point; no side-effect path around it |
| T8 | **Capability escalation** — action fabricates capabilities | Kernel integrity | Capabilities validated against the provisioned set; typed, not inferred from payload |
| T9 | **Approval bypass** — high-risk action runs without approval | Kernel integrity | Approval gate enforced in kernel; `RequireApproval` → no execution until an approval record exists |
| T10 | **Symlink swap / TOCTOU** during read-modify-write | Workspace integrity | Re-resolve paths after canonicalization; use open with no-follow where possible |
| T11 | **Evidence forgery** — forged `ExecutionResult` | Trust/replay | Results generated inside the kernel with identity + decision + timestamps; not trusted from adapters |
| T12 | **Denial of the sandbox** — sandbox misconfiguration (e.g., Landlock ABI gaps) | Host | Fail-closed: if OS isolation is unavailable, downgrade to tier-1-only and refuse process/network actions |

**Out of scope for the MVP threat model:** multi-tenant adversarial isolation of the
host (this is a single-user local tool), supply-chain of toolchain images (containers
are deferred), and network-hardened remote sandboxing (offline-first).

---

## 10. Selected approach (detail)

### 10.1 Tier 1 — Workspace confinement (MVP)

- The **`Workspace`** struct holds allow-listed roots; every `AgentAction` payload is
  resolved via `PathResolution`.
- Resolution: make absolute → canonicalize (resolve symlinks) → verify the result is
  lexically and physically inside an allowed root → reject otherwise.
- Applies to all `read_file` / `write_file` / `patch_file` / `git` (worktree) /
  `run_test` (cwd) operations. Nothing touches `..` or absolute paths outside the
  allow-list.
- Deterministic and testable in pure Rust with no OS-specific dependencies.

### 10.2 Tier 2 — OS-level isolation (process/network)

- A **`SandboxAdaptor`** trait abstracts per-OS confinement. The MVP ships a **no-op
  adaptor** (fail-closed when process/network execution is requested) and, on Linux, an
  **optional Landlock + seccomp + `setrlimit`** adaptor.
- When the adaptor is requested but unavailable, execution of process/network actions is
  **denied** (T12), preserving the "dry-run refuses privileged effects" invariant.
- Resource limits (timeout, memory, output bytes) are enforced at this layer.

### 10.3 Tier 3 — WASM capability execution (future)

- Optional and opt-in. Agent-supplied tools/probe code compiled to WASM run under WASI
  with only the explicitly granted directories and capabilities.
- Not required for the MVP; added when untrusted agent-code execution becomes a real
  need.
## 11. Rejected alternatives

| Alternative | Why rejected |
| --- | --- |
| **Path sandboxing alone** (3.1) | Fails to confine processes/network; insufficient for the `execute`/`git`/`run_test` roadmap. Kept as tier 1, not the whole answer. |
| **OS isolation alone** (3.2) | Strong but OS-specific and complex; no portable base; makes MVP and testing harder. Used as tier 2, not the sole mechanism. |
| **WASM/WASI as the primary sandbox** (3.3) | Cannot execute real native toolchains (compilers, git, test runners) as WASM; not a drop-in for the roadmap. Deferred to tier 3 for agent-code execution. |
| **Containers / lightweight VMs** (3.4) | Not viable on Android, heavy runtime + latency, poor offline fit, overkill for granular typed actions. Deferred to an optional heavy-worker path. |
| **Single monolith sandbox** | Rejected for layering: mixing confinement, OS isolation, and WASM into one mechanism increases coupling and weakens the portability guarantee. |

---

## 12. Future migration options

- **Tier-2 hardening per OS:** add Linux namespaces (mount/net) for deeper isolation;
  macOS hardened runtime + App Sandbox; Windows AppContainer hardening. All behind the
  `SandboxAdaptor` trait — no policy-layer changes.
- **Tier-3 WASM:** introduce a WASI runtime (Wasmtime/Wasmer) as an optional dependency
  behind a Cargo feature flag for executing untrusted agent tools.
- **Containers/VM workers:** add an optional "heavy build worker" path (OCI or
  Firecracker) for CI-grade workloads, gated by capability policy, on desktop/server
  targets — not on Android.
- **Network policy:** once network execution is authorized, extend Landlock/sandbox
  profiles to specific ports/hosts (Landlock ABI ≥ 4) behind the same adaptor.
- **Remote/federated workers:** expose the same typed protocol + evidence model to Go
  workers so offloading execution to a remote sandbox is a transport change, not an API
  change.
## 13. Smallest validation implementation

A minimal, dependency-light implementation that proves the architectural decision
**without filesystem mutation or full process execution**.

### 13.1 Scope

1. **`Workspace` + `resolve_path`** implementing tier-1 path confinement, with unit tests
   for allow, deny, absolute, `..`, and symlink/escape cases (no real FS writes; use a
   temp dir in tests).
2. **`ExecutionError`** and a real `ExecutionStatus` / `ExecutionResult` aligned to
   `execution-result.schema.json` (fixing the current schema drift).
3. **`SandboxAdaptor` trait** with a **no-op / fail-closed** implementation and a
   `SandboxUnavailable` error — proving the T12 fail-closed path.
4. **`execute(...)`** that runs only when passed a pre-gated `ExecutableAction` and
   returns schema-conformant evidence, wired to the existing `evaluate_policy` +
   `dry_run` so the full chain
   `Agent → Action → Validation → Capability → Policy → Approval → Execution → Evidence`
   is represented.

### 13.2 Acceptance criteria

- `cargo build --workspace` and `cargo test --workspace` pass (including fixing the
  `serde_json` dependency so the build gate is green).
- Unit tests demonstrate: allowed/denied path resolution; fail-closed behavior when the
  sandbox adaptor is unavailable; `ExecutionResult` serializes to schema-conformant JSON;
  no privileged side effect is possible without a pre-gated `ExecutableAction`.
- No filesystem mutation, process spawning, or network access is added.

### 13.3 Why this validates the decision

It proves every load-bearing property of the hybrid design with minimal surface:
(1) tier-1 confinement works and is testable cross-platform, (2) tier-2 fail-closed
behavior is sound, and (3) the evidence contract is real and schema-conformant — all
before any dangerous execution capability or new language is introduced.

---

## 14. References

- Repository assessment: `docs/architecture/repository-assessment.md`
- Agent action lifecycle: `docs/architecture/agent-protocol.md`
- Protocol contracts: `protocols/agent-action.schema.json`,
  `protocols/execution-result.schema.json`, `protocols/task.schema.json`
- Linux Landlock: kernel docs (unprivileged LSM, filesystem + network rules)
- macOS sandbox: `sandbox-exec(1)` / `sandbox_init(3)`
- WASI: Bytecode Alliance WASI intro (capability-based filesystem/network/clock)
- Wasmer: sandboxed WASM runtime with WASI support
   not just side effects.