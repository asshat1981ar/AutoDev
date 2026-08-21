# Mistral Connector Control Plane Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a dependency-free, Git-authoritative Mistral Connector reconciler that validates connector manifests, produces deterministic safe diffs, detects tool drift, and can apply approved create/update operations without handling secrets or deletions.

**Architecture:** New JSON-subset YAML manifests under `connectors/` feed a pure Python reconciliation core in `scripts/mistral_connector_sync.py`. The core separates validation, diff planning, HTTP transport, mutation application, and tool-snapshot comparison so behavior is unit-testable without network access. Live Mistral calls require `MISTRAL_API_KEY`; default operation is dry-run and deletion is intentionally absent.

**Tech Stack:** Python 3.10/3.11 standard library, `unittest`, GitHub Actions, Mistral Studio Connector REST API.

**Spec:** `docs/superpowers/specs/2026-08-21-mistral-connector-control-plane-design.md`

## Global Constraints

- No root `pyproject.toml`, `requirements.txt`, or package manager changes.
- Python implementation remains standard-library only.
- ForgeCore remains the sole trusted execution authority for AutoDev effects.
- No raw credentials or tokens in manifests, logs, snapshots, fixtures, or commits.
- No implicit Connector deletion/pruning.
- Default CLI behavior is non-mutating.
- `shared_org` mutations require an explicit elevated opt-in.
- OAuth token injection is not automated.
- Verification claims require fresh CI/repository evidence.

---

### Task 1: Lock connector contracts with failing tests

**Files:**
- Create: `tests/test_mistral_connector_sync.py`

**Interfaces:**
- Consumes: Python standard library only.
- Produces expected interfaces: `load_manifest`, `validate_manifest`, `plan_reconciliation`, `sanitize`, `diff_tools`, `MistralConnectorClient`, `apply_plan`.

- [ ] **Step 1: Write failing behavioral tests**

Cover:

```python
from scripts.mistral_connector_sync import (
    ManifestError,
    MistralConnectorClient,
    apply_plan,
    diff_tools,
    load_manifest,
    plan_reconciliation,
    sanitize,
    validate_manifest,
)
```

Required cases:

```text
valid managed MCP manifest
invalid connector name
invalid visibility
secret-like manifest field rejected
featured connector becomes EXTERNAL
missing remote becomes CREATE
matching remote becomes NOOP
changed remote becomes UPDATE
shared_org write becomes BLOCKED without elevation
apply refuses BLOCKED/EXTERNAL actions
apply never synthesizes delete
tool snapshot reports added/removed/changed
sanitizer redacts authorization/api-key/token values
HTTP client serializes documented create/update/list/list-tools requests
```

- [ ] **Step 2: Push the red test commit to an isolated feature branch**
- [ ] **Step 3: Open a draft PR to `main` so CI runs**
- [ ] **Step 4: Verify the Python CI job fails because `scripts.mistral_connector_sync` is absent**

### Task 2: Implement manifest validation and deterministic planning

**Files:**
- Create: `scripts/mistral_connector_sync.py`
- Create: `connectors/registry.yaml`
- Create: `connectors/deepwiki.yaml`
- Create: `connectors/github.yaml`
- Create: `connectors/linear.yaml`
- Test: `tests/test_mistral_connector_sync.py`

**Interfaces:**
- Consumes tests from Task 1.
- Produces:

```python
class ManifestError(ValueError): ...
def load_manifest(path: str | Path) -> dict: ...
def validate_manifest(data: dict) -> dict: ...
def sanitize(value): ...
def plan_reconciliation(desired: dict, remote: dict | None, *, allow_org_shared: bool = False) -> dict: ...
def diff_tools(previous: list[dict], current: list[dict]) -> dict: ...
```

- [ ] **Step 1: Run Task 1 tests and confirm RED**
- [ ] **Step 2: Implement the minimal validation/planning core**
- [ ] **Step 3: Add JSON-subset YAML manifests**
- [ ] **Step 4: Run focused unit tests and require GREEN**
- [ ] **Step 5: Refactor without changing behavior**

### Task 3: Implement Mistral REST adapter and apply boundary

**Files:**
- Modify: `scripts/mistral_connector_sync.py`
- Test: `tests/test_mistral_connector_sync.py`

**Interfaces:**
- Consumes `urllib.request` compatible injectable transport.
- Produces:

```python
class MistralConnectorClient:
    def list_connectors(self, *, page_size: int = 100) -> list[dict]: ...
    def get_connector(self, connector_id_or_name: str) -> dict: ...
    def create_connector(self, desired: dict) -> dict: ...
    def update_connector(self, connector_id: str, changes: dict) -> dict: ...
    def list_tools(self, connector_id_or_name: str, *, refresh: bool = False, pretty: bool = True) -> list[dict]: ...

def apply_plan(client, action: dict, *, allow_org_shared: bool = False) -> dict: ...
```

- [ ] **Step 1: Add/confirm failing HTTP serialization tests**
- [ ] **Step 2: Implement request encoding, pagination, and response normalization**
- [ ] **Step 3: Implement safe `CREATE`/`UPDATE` application only**
- [ ] **Step 4: Prove `EXTERNAL`, `BLOCKED`, `NOOP`, and unknown actions cannot mutate**
- [ ] **Step 5: Run unit tests**

### Task 4: Add CLI dry-run and tool drift evidence

**Files:**
- Modify: `scripts/mistral_connector_sync.py`
- Create: `docs/integrations/mistral-connectors.md`
- Test: `tests/test_mistral_connector_sync.py`

**Interfaces:**
- Produces CLI commands:

```text
python scripts/mistral_connector_sync.py validate connectors/registry.yaml
python scripts/mistral_connector_sync.py plan connectors/deepwiki.yaml --remote-file tests/fixtures/mistral/remote-empty.json
python scripts/mistral_connector_sync.py diff-tools OLD.json NEW.json
```

Live operations may be added as:

```text
python scripts/mistral_connector_sync.py live-plan connectors/deepwiki.yaml
python scripts/mistral_connector_sync.py apply connectors/deepwiki.yaml
```

but `apply` must require `MISTRAL_API_KEY`, explicit `--apply`, and elevated opt-in for organization-shared state.

- [ ] **Step 1: Add CLI tests for non-mutating commands**
- [ ] **Step 2: Implement CLI parser and stable JSON output**
- [ ] **Step 3: Document supported lifecycle and approval boundary**
- [ ] **Step 4: Run unit tests and harness drift**

### Task 5: Exact-head verification and review

**Files:**
- No new implementation files unless verification identifies a defect.

**Interfaces:**
- Consumes exact PR head SHA.
- Produces verification evidence only.

- [ ] **Step 1: Fetch exact PR head**
- [ ] **Step 2: Require Python 3.10 and 3.11 CI jobs to pass**
- [ ] **Step 3: Require harness drift/reproducible job to pass**
- [ ] **Step 4: Inspect complete PR diff for unintended files and secret leakage**
- [ ] **Step 5: Inspect independent review findings if available; do not claim CodeRabbit review unless evidence exists**
- [ ] **Step 6: Record actual outcome in the plan and stop before any live Mistral Connector mutation**

## Progress

- Task 1: pending
- Task 2: pending
- Task 3: pending
- Task 4: pending
- Task 5: pending

## Surprises & Discoveries

- Local container cannot resolve `github.com`; implementation and TDD verification must use the connected GitHub API and GitHub Actions rather than a local clone.
- Existing `feat/vibe-mcp-control-plane` is a distinct unmerged MCP-server integration and must not be used as the implementation branch for Mistral Connector provisioning.

## Decision Log

- Use strict JSON syntax in `.yaml` files so manifests remain valid YAML while AutoDev can parse them with stdlib `json` and avoid a new dependency/ADR.
- Implement against documented REST semantics behind an injectable transport instead of adding the `mistralai` SDK dependency to the root Python fabric.
- Treat featured Mistral Connectors as unmanaged external resources in this slice.

## Outcomes & Retrospective

Not yet populated. Completion evidence must come from exact-head CI and diff inspection.