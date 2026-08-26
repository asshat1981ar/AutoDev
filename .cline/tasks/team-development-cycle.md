# Task: Team Development Cycle

**ID**: team-development-cycle
**Type**: Team Coordination / Development
**Priority**: High
**Estimated Duration**: Varies (per cycle)
**Required Agents**: Config Manager, Architect, Builder, Reviewer, Verifier
**Worktree**: Any (primary or team worktrees)

## Objective

Execute a complete team development cycle using Cline fabric, WEX protocol, and centralized configuration. This task coordinates parallel development across multiple worktrees while maintaining harness compliance.

## Prerequisites

- WEX protocol initialized (`python scripts/wex-cli.py init`)
- Centralized configuration in place (`config/` directory)
- Cline agents available (Config Manager, Architect, Builder, etc.)
- Git worktrees set up for parallel development
- WEX dashboard operational

## Inputs

Required inputs (provide when starting this task):
- `cycle_name`: Name of the development cycle (e.g., "Kotlin MPP Integration")
- `worktrees`: List of worktrees participating (e.g., ["primary", "team-1"])
- `scope`: Development scope (e.g., "Kotlin modules", "Rust ForgeCore")

Optional inputs:
- `duration`: Expected cycle duration in days
- `milestones`: Key milestones for this cycle
- `dependencies`: Other cycles this depends on

## Steps

### Step 1: Cycle Planning (S1 - Plan Mode)

**Agent**: Architect + Config Manager
**Action**: Define cycle scope, milestones, and worktree assignments

```bash
# Review current state
python scripts/wex-cli.py dashboard
python config/validate.py
python scripts/check_harness_drift.py

# Check path reservations
python scripts/wex-cli.py check-collisions
```

**Acceptance Criteria**:
- [ ] Cycle scope documented
- [ ] Worktrees assigned to specific tasks
- [ ] Path reservations updated for all participants
- [ ] No collision warnings
- [ ] All configurations validated

**Artifacts**:
- Cycle plan document (markdown)
- Updated reservations.json
- Initial evidence record

---

### Step 2: Worktree Activation

**Agent**: All participating agents
**Action**: Activate each worktree with proper identity and task

For each worktree in `worktrees`:

```bash
cd .worktrees/<worktree-name>

# Initialize WEX (if not done)
python ../scripts/wex-cli.py init

# Emit heartbeat with task
python ../scripts/wex-cli.py heartbeat \
  --task "<cycle_name>: <specific-task>" \
  --status active \
  --milestone "<initial-milestone>"
```

**Acceptance Criteria**:
- [ ] All worktrees show as "active" in dashboard
- [ ] Each worktree has correct task and milestone
- [ ] Heartbeats within interval (300s)

**Artifacts**:
- Updated manifest.json
- Updated heartbeat.json per worktree

---

### Step 3: Parallel Development

**Agent**: Builder agents in each worktree
**Action**: Implement features with centralized config

For each worktree:

```bash
# Reserve paths for this work
python ../scripts/wex-cli.py reserve <path1>
python ../scripts/wex-cli.py reserve <path2>

# Verify no collisions
python ../scripts/wex-cli.py check-collisions

# Use centralized config
# Example: For Kotlin, reference config/kotlin/gradle.properties
# Example: For Rust, reference config/defaults/clippy.toml

# Run validation before commits
python ../config/validate.py
python ../scripts/check_harness_drift.py

# Stage changes
git add -A

# Commit with proper prefix
# feat(kotlin): <description>
# fix(config): <description>
# chore(ci): <description>
```

**Acceptance Criteria**:
- [ ] All path reservations respected
- [ ] No collision warnings during development
- [ ] All changes pass config validation
- [ ] All changes pass harness drift check
- [ ] Commit messages follow conventions

**Artifacts**:
- Git commits per worktree
- Evidence records per significant change

---

### Step 4: Evidence Generation

**Agent**: Builder + Verifier
**Action**: Generate evidence for all development work

```bash
# Create evidence record (manual or via hook)
cat > .wex/evidence-outbox/<evidence-id>.json << 'EOF'
{
  "evidence_id": "<unique-id>-<date>",
  "type": "development",
  "result": "passed|in_progress|failed",
  "description": "<what was done>",
  "worktree": "<worktree-id>",
  "branch": "<branch-name>",
  "checks": {
    "config_validation": "pass (0 errors, N warnings)",
    "harness_drift": "pass",
    "path_reservation": "<paths reserved>",
    "collision_check": "pass (no collisions)"
  },
  "files_modified": ["<list of files>"],
  "config_used": ["<list of config files>"],
  "wex_version": "1.0.0"
}
EOF

# Push evidence to registry (from any worktree)
# Note: Use python script to copy from outbox to registry
```

**Acceptance Criteria**:
- [ ] Evidence records created for all significant work
- [ ] Evidence pushed to registry
- [ ] Evidence includes all required checks
- [ ] Evidence references correct config files

**Artifacts**:
- Evidence records in registry
- Updated evidence-exchange directory

---

### Step 5: Integration & Verification

**Agent**: Verifier + Reviewer
**Action**: Verify all worktree changes integrate correctly

```bash
# From primary worktree
python scripts/wex-cli.py dashboard
python config/validate.py --strict
python scripts/check_harness_drift.py

# Check all evidence
python scripts/wex-cli.py pull-evidence  # If needed

# Run full verification
# (Rust, Kotlin, Python as applicable)
```

**Acceptance Criteria**:
- [ ] All worktrees pass validation
- [ ] No harness drift introduced
- [ ] All evidence records show "passed"
- [ ] No collisions detected
- [ ] Integration tests pass (if applicable)

**Artifacts**:
- Integration verification report
- Updated WEX dashboard

---

### Step 6: Cycle Completion

**Agent**: Architect + Config Manager
**Action**: Close out the development cycle

```bash
# Update primary heartbeat with completion
python scripts/wex-cli.py heartbeat \
  --task "<cycle_name> - COMPLETED" \
  --status completed \
  --milestone "All worktrees integrated"

# Generate final evidence
# (Create cycle completion evidence)

# Commit all changes
git add -A
git commit -m "feat(cycle): <cycle_name> completed"

# Push to remote (if applicable)
git push origin <branch>
```

**Acceptance Criteria**:
- [ ] All worktrees show as "completed" or "active"
- [ ] Final evidence record generated
- [ ] All changes committed
- [ ] No validation errors
- [ ] No harness drift

**Artifacts**:
- Cycle completion evidence
- Git commits pushed to remote
- Updated WEX dashboard

---

## Success Criteria

The team development cycle is considered successful when:

1. ✅ All participating worktrees are active and healthy
2. ✅ No path collisions occurred during development
3. ✅ All changes pass config validation (0 errors)
4. ✅ All changes pass harness drift check
5. ✅ Evidence records generated for all significant work
6. ✅ All evidence shows "passed" or "completed" status
7. ✅ Changes are committed with proper conventions

## Rollback Procedure

If the cycle must be rolled back:

1. Revert git commits in affected worktrees
2. Clear reservations that are no longer valid
3. Remove evidence records for rolled-back work
4. Update heartbeats to reflect current state
5. Document rollback in evidence

## Escalation Path

1. **Configuration questions**: Config Manager (S0)
2. **Architecture questions**: Architect (S1)
3. **Collision conflicts**: All agents - stop and resolve
4. **Harness violations**: Config Manager + Security - immediate review
5. **WEX protocol issues**: Primary worktree - restore from git

## Related Tasks

- `config-audit`: Validate configurations before cycle start
- `config-update`: Update configurations during cycle
- `repo-recon`: Recon repository before major changes

## References

- [WEX Protocol Documentation](docs/architecture/WEX_PROTOCOL.md)
- [Configuration Architecture](docs/architecture/CONFIG_ARCHITECTURE.md)
- [Team Collaboration Guide](TEAM_COLLABORATION.md)
- [AGENTS.md Harness Rules](../AGENTS.md)
