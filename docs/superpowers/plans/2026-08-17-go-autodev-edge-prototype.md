# Go AutoDev Edge Prototype Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a removable Go connectivity prototype that proves or rejects Go as AutoDev's external MCP/provider transport boundary without creating another policy engine, orchestrator, or trusted executor.

**Architecture:** Add `go/autodev-edge` as a separate optional process. Use the official `github.com/modelcontextprotocol/go-sdk` for MCP lifecycle/transport semantics, wrap it behind AutoDev-owned interfaces, expose only normalized connectivity observations to the Rust control plane, and compare process-boundary options plus a Rust/Tokio equivalent before production adoption.

**Tech Stack:** Go 1.26.5 toolchain, `go 1.25.0` module language floor, official MCP Go SDK v1.7.0, standard `context`, `net/http`, `slog`, `testing`, and `pprof`/runtime metrics where needed.

## Global Constraints

- Go Edge is not a ForgeCore replacement.
- Go Edge cannot mint capabilities, approvals, task completion, execution evidence acceptance, or repository mutations.
- No JNI, FFI, cgo, embedded Go mobile library, database, distributed-worker scheduler, or generic public AutoDev server in this plan.
- Support MCP protocol `2026-07-28` through the official SDK and preserve SDK-supported negotiation with older protocol revisions.
- Local HTTP control surfaces bind loopback only and require a generated local bearer token; tokens must never enter logs.
- Every goroutine has context-derived cancellation or a bounded lifecycle owner.
- Queue/channel capacities are explicit and tested; no unbounded fan-out.

---

## File Structure

- Create `go/autodev-edge/go.mod` and `go.sum`.
- Create `go/autodev-edge/cmd/autodev-edge/main.go` — process bootstrap only.
- Create `go/autodev-edge/internal/config/config.go` — validated runtime configuration.
- Create `go/autodev-edge/internal/edge/types.go` — normalized AutoDev connectivity types.
- Create `go/autodev-edge/internal/edge/manager.go` — bounded lifecycle manager.
- Create `go/autodev-edge/internal/mcpclient/client.go` — official SDK adapter.
- Create `go/autodev-edge/internal/control/server.go` — loopback observation/control prototype.
- Create `go/autodev-edge/internal/control/auth.go` — local bearer-token verification.
- Create `go/autodev-edge/internal/telemetry/collector.go` — bounded health observations.
- Create tests beside each package.
- Create `go/autodev-edge/testdata/` for canonical public protocol and MCP fixtures.
- Modify `.github/workflows/ci.yml` only after the prototype passes local tests.

### Task 1: Establish the Go module and lifecycle-safe process shell

**Interfaces:**
- `config.Load(getenv func(string) string) (Config, error)`.
- `edge.Manager.Run(ctx context.Context) error`.
- `edge.Manager.Close() error` must be idempotent.

- [ ] **Step 1: Create `go.mod`**

```go
module github.com/asshat1981ar/AutoDev/go/autodev-edge

go 1.25.0

require github.com/modelcontextprotocol/go-sdk v1.7.0
```

Use Go 1.26.5 in development/CI; the lower module language version intentionally preserves compatibility with the currently supported preceding major line.

- [ ] **Step 2: Write failing config tests for invalid bind host, zero/negative capacities, empty upstream name, and missing local token when HTTP control is enabled.**
- [ ] **Step 3: Implement immutable validated `Config` values; default local bind must be `127.0.0.1:8791`.**
- [ ] **Step 4: Implement `main` with `signal.NotifyContext`, one root context, `defer manager.Close()`, and `slog` with token redaction.**
- [ ] **Step 5: Run**

```bash
cd go/autodev-edge
go test ./...
go vet ./...
```

Expected: PASS.

- [ ] **Step 6: Commit `feat(go): scaffold cancellable AutoDev Edge`.**

### Task 2: Define normalized connectivity observations

**Files:** `internal/edge/types.go`, tests, `testdata/connectivity-status.json` copied from canonical public v1 fixture.

**Interfaces:**

```go
type ConnectionState string

const (
	ConnectionDisconnected ConnectionState = "disconnected"
	ConnectionConnecting   ConnectionState = "connecting"
	ConnectionReady        ConnectionState = "ready"
	ConnectionDegraded     ConnectionState = "degraded"
)

type ConnectivityStatus struct {
	SchemaVersion string          `json:"schema_version"`
	SourceID      string          `json:"source_id"`
	Kind          string          `json:"kind"`
	State         ConnectionState `json:"state"`
	Protocol      string          `json:"protocol"`
	LatencyMS     *int64          `json:"latency_ms"`
	ObservedAt    string          `json:"observed_at"`
	Detail        string          `json:"detail"`
}
```

No field may carry AutoDev capability/approval material.

- [ ] **Step 1: Write JSON fixture round-trip tests and malformed enum tests.**
- [ ] **Step 2: Implement types and explicit validation.**
- [ ] **Step 3: Run `go test ./internal/edge -race`.**
- [ ] **Step 4: Commit `feat(go): add normalized connectivity observations`.**

### Task 3: Wrap the official MCP Go SDK behind an AutoDev transport interface

**Interfaces:**

```go
type Client interface {
	Connect(ctx context.Context, upstream Upstream) (Session, error)
}

type Session interface {
	ListTools(ctx context.Context) ([]ToolSummary, error)
	CallTool(ctx context.Context, name string, arguments map[string]any) (ToolResult, error)
	Close() error
}
```

`mcpclient.SDKClient` implements these interfaces using `github.com/modelcontextprotocol/go-sdk/mcp`. AutoDev packages outside `internal/mcpclient` must not depend directly on SDK session types.

- [ ] **Step 1: Add mock-interface tests for cancellation before connect, timeout, tool discovery, tool error, and close.**
- [ ] **Step 2: Implement Streamable HTTP and command/stdio upstream selection with the SDK's transport implementations.**
- [ ] **Step 3: Preserve negotiated protocol version as observation metadata; do not branch core AutoDev behavior on SDK-specific internal types.**
- [ ] **Step 4: Use per-request contexts; no `context.Background()` inside library code.**
- [ ] **Step 5: Run `go test ./internal/mcpclient -race`.**
- [ ] **Step 6: Commit `feat(go): adapt official MCP client SDK`.**

### Task 4: Add bounded connection manager, retry, and backpressure semantics

**Interfaces:**
- Maximum concurrent upstream sessions defaults to 16.
- Observation channel capacity defaults to 256.
- Retry delays: 250 ms, 500 ms, 1 s, 2 s, 5 s maximum with cancellation-aware timers.
- When observation buffer is full, replace/coalesce stale status for the same source rather than blocking a transport indefinitely.

- [ ] **Step 1: Add race-enabled tests creating 64 requested upstreams and assert active sessions never exceed configured limit.**
- [ ] **Step 2: Add deterministic retry tests using an injected clock/timer function.**
- [ ] **Step 3: Add backpressure tests ensuring a stalled observer cannot leak goroutines or grow memory without bound.**
- [ ] **Step 4: Implement manager with `errgroup`-style structured ownership using standard contexts/channels; if adding `golang.org/x/sync/errgroup`, justify and pin it in the task commit. Prefer standard library for prototype simplicity.**
- [ ] **Step 5: Run `go test ./... -race`.**
- [ ] **Step 6: Commit `feat(go): add bounded edge connection lifecycle`.**

### Task 5: Implement authenticated loopback control/observation prototype

**Interfaces:**
- `GET /health` returns process health only.
- `GET /api/v1/connectivity` returns normalized `ConnectivityStatus[]`.
- No objective mutation, approval, execution, repository, or ForgeCore effect endpoint exists.

Bearer-token verification must use `crypto/subtle.ConstantTimeCompare` over SHA-256 digests or an equivalently constant-time comparison path; never log the supplied header.

- [ ] **Step 1: Write tests asserting non-loopback configured bind is rejected by `Config` for prototype mode.**
- [ ] **Step 2: Add HTTP tests for missing token = 401, wrong token = 401, correct token = 200, and response contains only public connectivity fields.**
- [ ] **Step 3: Implement the server with `http.Server` read-header timeout, idle timeout, bounded handler work, and shutdown using the root context.**
- [ ] **Step 4: Run `go test ./internal/control -race`.**
- [ ] **Step 5: Commit `feat(go): add authenticated loopback edge API`.**

### Task 6: Prototype the alternative stdio process boundary

Build a small Rust-side test harness or standalone integration fixture under `crates/autodev-server/tests/edge_stdio.rs` that starts the Go binary as a child only for the test and exchanges newline-delimited observation messages. Do not integrate it into production server startup.

- [ ] Verify child cancellation/kill cleanup.
- [ ] Verify malformed stdout lines fail closed and are not forwarded as observations.
- [ ] Compare local bearer HTTP vs stdio on implementation complexity, process ownership, cancellation, telemetry, and failure isolation.
- [ ] Record the result under `docs/benchmarks/go-edge/process-boundary.md`.

### Task 7: Collect Go runtime/transport benchmark evidence

Use fixed mock upstreams and record:

- cold process startup;
- idle RSS;
- 1/8/16 concurrent MCP sessions;
- reconnect latency after forced disconnect;
- sustained observation delivery at 100, 1,000, and 10,000 events/minute;
- CPU/RSS under load;
- goroutine count before/after teardown;
- binary size;
- `go test -race` result.

Store raw data plus `go version`, OS/kernel, CPU and commit SHA under `docs/benchmarks/go-edge/`.

### Task 8: Compare against a minimal Rust/Tokio equivalent

Implement only enough Rust benchmark fixture code to exercise the same mock connection/retry/fan-out workload; do not create a second production service. Compare implementation size, failure semantics, throughput/latency, RSS, cancellation, and maintenance surface.

If Go does not offer a meaningful capability/isolation/maintenance advantage, mark the production adoption gate rejected and retain only the benchmark/spec evidence.

### Task 9: CI gate after prototype is green

Add a `go` job to `.github/workflows/ci.yml` using Go 1.26.5 and run:

```bash
cd go/autodev-edge
gofmt -w .
git diff --exit-code
go vet ./...
go test ./... -race
go build ./cmd/autodev-edge
```

Keep existing Rust/Kotlin/Python jobs unchanged.

## Done Gate

The Go prototype is complete only when the official MCP SDK is isolated behind AutoDev interfaces, all concurrency is bounded/cancellable, loopback control is authenticated, race tests pass, process-boundary and Rust/Tokio comparisons are recorded, and Go still possesses zero trusted repository authority.