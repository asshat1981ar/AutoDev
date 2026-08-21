# Cline Onboarding for Team Development

This guide helps team members onboard with Cline fabric for AutoDev's next development cycle.

## 🎯 Quick Start (5 minutes)

### 1. Verify Cline Setup

```bash
# From repository root
cd /path/to/AutoDev

# Check Cline fabric exists
ls -la .cline/
# Should show: agents/, config/, hooks/, mcp/, plugins/, rules/, skills/, tasks/

# Validate configuration
python config/validate.py
# Expected: 0 errors, 5 optional warnings
```

### 2. Initialize WEX in Your Worktree

```bash
# If you're in a worktree (not primary)
cd .worktrees/<your-worktree>
python ../scripts/wex-cli.py init

# Emit your first heartbeat
python ../scripts/wex-cli.py heartbeat \
  --task "Onboarding - Development Cycle" \
  --status active \
  --milestone "Starting development"
```

### 3. Check Team Status

```bash
# From primary worktree
cd /path/to/AutoDev
python scripts/wex-cli.py dashboard
# Should show your worktree as "active"
```

---

## 🏗️ Cline Infrastructure Overview

### Directory Structure

```
.cline/
├── README.md                    # Cline fabric documentation
├── CLINE_ONBOARDING.md          # This file
│
├── agents/                      # Agent definitions
│   ├── architect.md             # S1: Architecture planning
│   ├── builder.md               # S0: Implementation
│   ├── config-manager.md        # S1: Configuration custodian
│   ├── reviewer.md              # S0: Code review
│   ├── scout.md                 # S0: Reconnaissance
│   ├── security.md              # S0: Security review
│   └── verifier.md              # S0: Verification
│
├── config/                     # Cline configuration
│   ├── capabilities.json       # Agent capabilities
│   ├── permissions.json        # Permission rules
│   └── policies/               # Policy definitions
│       ├── default.yaml
│       └── production.yaml
│
├── hooks/                      # Pre/post action hooks
│   ├── README.md
│   ├── hooks.json              # Hook configuration
│   ├── post_tool_use.py
│   ├── pre_compact.py
│   ├── pre_config_change.py    # Config validation hook
│   ├── pre_tool_use.py
│   └── task_start.py
│
├── mcp/                        # MCP server profiles
│   └── profiles.json
│
├── plugins/                    # Local plugins
│   └── project-fabric/
│       ├── plugin.json
│       └── tools.py
│
├── rules/                      # Agent behavior rules
│   ├── 00-recon.md
│   ├── 10-safety.md
│   ├── 20-verification.md
│   └── 30-context.md
│
├── skills/                     # Specialized skills
│   ├── architecture-design/
│   ├── debugging/
│   ├── release-readiness/
│   ├── repo-recon/
│   ├── research-validation/
│   ├── security-review/
│   ├── test-strategy/
│   ├── vertical-slice/
│   └── ...
│
└── tasks/                      # Structured tasks
    ├── config-audit.md          # Configuration audit
    ├── config-update.md         # Configuration update
    └── team-development-cycle.md # Team dev cycle
```

### Available Agents

| Agent | Role | Capability | When to Use |
|-------|------|------------|--------------|
| **Config Manager** | Configuration custodian | S1 | Config changes, validation |
| **Architect** | Architecture planning | S1 | Cross-component changes |
| **Builder** | Implementation | S0 | Direct code changes |
| **Reviewer** | Code review | S0 | PR reviews, quality gates |
| **Verifier** | Verification | S0 | Testing, validation |
| **Scout** | Reconnaissance | S0 | Repository exploration |
| **Security** | Security review | S0 | Security-sensitive changes |

---

## 🚀 Development Workflow with Cline

### Step 1: Plan (S1 - Plan Mode)

**For cross-component or architectural changes:**

```
Architect, plan the Kotlin MPP integration with centralized config
```

**For configuration changes:**

```
Config Manager, plan the Rust toolchain version bump
```

**Required inputs:**
- Scope of changes
- Affected components
- Timeline/milestones
- Risk assessment

### Step 2: Reserve Paths

Before modifying any files, reserve your paths to prevent collisions:

```bash
python scripts/wex-cli.py reserve <path-to-file-or-dir>

# Example
python scripts/wex-cli.py reserve kotlin/mpp-core/src
python scripts/wex-cli.py reserve config/kotlin/gradle.properties
```

**Verify no collisions:**

```bash
python scripts/wex-cli.py check-collisions
# Expected: "No collisions detected."
```

### Step 3: Implement

Use the **Builder** agent for direct implementation:

```
Builder, update mpp-core/build.gradle.kts to use config/kotlin/gradle.properties
```

**Or work directly:**

```bash
# Edit files
ingleton/mpp-core/build.gradle.kts

# Validate before commit
python config/validate.py
python scripts/check_harness_drift.py
```

### Step 4: Validate

Use the **Verifier** agent:

```
Verifier, validate the Kotlin MPP changes against harness rules
```

**Manual validation:**

```bash
python config/validate.py --strict
python scripts/check_harness_drift.py
```

### Step 5: Review

Use the **Reviewer** agent:

```
Reviewer, review the changes in kotlin/mpp-core for config compliance
```

### Step 6: Commit

Follow commit conventions:

```bash
git add -A
git commit -m "feat(kotlin): Integrate centralized gradle.properties"
```

### Step 7: Generate Evidence

Create an evidence record for your work:

```bash
cat > .wex/evidence-outbox/<evidence-id>.json << 'EOF'
{
  "evidence_id": "<your-name>-<work-description>-<date>",
  "type": "development",
  "result": "passed",
  "description": "<what you did>",
  "worktree": "<your-worktree-id>",
  "branch": "<your-branch>",
  "reserved_paths": ["<paths-you-reserved>"],
  "files_modified": ["<files-you-changed>"],
  "checks": {
    "config_validation": "pass (0 errors, N warnings)",
    "harness_drift": "pass",
    "path_reservation": "<your-paths>",
    "collision_check": "pass (no collisions)"
  },
  "config_used": ["<config-files-referenced>"],
  "wex_version": "1.0.0"
}
EOF
```

Then push to registry:

```bash
# From primary worktree
cp .worktrees/<your-worktree>/.wex/evidence-outbox/*.json .worktrees/.registry/evidence-exchange/
```

---

## 📋 Task Catalog

### Available Tasks

| Task ID | Purpose | Duration | Required Agents |
|---------|---------|----------|------------------|
| `config-audit` | Audit all configurations | 30-60 min | Config Manager, Security, Verifier |
| `config-update` | Update configuration files | 15-45 min | Config Manager, Builder |
| `team-development-cycle` | Full team dev cycle | Varies | All agents |
| `repo-recon` | Map repository structure | 15-30 min | Scout |

### Starting a Task

```
# Direct task invocation
Task: config-audit

# Or with parameters
Task: config-update component=kotlin file=gradle.properties change="bump ktlint to 12.1.2"
```

---

## 🔍 Common Patterns

### Pattern 1: Configuration Change

```
1. Config Manager, audit all configurations
2. Config Manager, plan the version bump
3. Builder, update config/kotlin/gradle.properties
4. Verifier, validate the change
5. Reviewer, review the change
6. Generate and push evidence
```

### Pattern 2: Feature Development

```
1. Architect, plan the feature implementation
2. Reserve paths for your work
3. Builder, implement the feature
4. Verifier, validate the implementation
5. Reviewer, review the code
6. Commit with proper message
7. Generate and push evidence
```

### Pattern 3: Cross-Component Change

```
1. Architect, plan the cross-component change (S1)
2. Config Manager, review config implications
3. Reserve paths in all affected worktrees
4. Builder, implement changes in each worktree
5. Verifier, validate each worktree
6. Reviewer, review all changes
7. Generate and push evidence from each worktree
```

---

## 🛡️ Safety & Quality Gates

### Always Run Before Commit

```bash
# From your worktree
python ../config/validate.py
python ../scripts/check_harness_drift.py
python ../scripts/wex-cli.py check-collisions
```

### Trust Boundaries (CRITICAL)

❌ **NEVER** bypass these rules:
- No root `Cargo.toml`, `package.json`, `pyproject.toml` without ADR
- No `AuthorizationGrant` creation outside ForgeCore
- No direct process execution bypassing Workspace confinement
- No secrets in configuration files
- No fail-open behavior in public adapters

✅ **ALWAYS** verify:
- All configs match AGENTS.md specifications
- All tool versions match CI workflow definitions
- Kotlin `commonMain` purity (no platform types)
- CLI authority boundary (no ForgeCore execution)

### Path Reservation Rules

- Reserve paths **before** starting work
- Check for collisions **before** each commit
- Release reservations when work is complete or abandoned
- One worktree per path (no overlapping reservations)

---

## 💡 Tips & Tricks

### 1. Quick Status Check

```bash
# From any worktree - check all team status
python ../scripts/wex-cli.py dashboard

# Check your worktree's reservations
python ../scripts/wex-cli.py list

# Check for conflicts
python ../scripts/wex-cli.py check-collisions
```

### 2. Finding the Right Agent

| Need | Agent |
|------|-------|
| "How does this config work?" | Config Manager |
| "Should we change this?" | Architect |
| "Implement this feature" | Builder |
| "Does this look right?" | Reviewer |
| "Test this" | Verifier |
| "Is this secure?" | Security |
| "What's in this repo?" | Scout |

### 3. Config Validation Shortcuts

```bash
# Quick validation
python config/validate.py

# Strict (warnings as errors)
python config/validate.py --strict

# Verbose output
python config/validate.py --verbose

# Harness drift check
python scripts/check_harness_drift.py
```

### 4. Working Across Worktrees

```bash
# List all worktrees
git worktree list

# Switch to a worktree
cd .worktrees/team-1

# Work on files (they're linked to the same repo)
# Changes in one worktree appear in all worktrees
# But each worktree can be on a different branch
```

---

## 🐛 Troubleshooting

### "WEx not initialized"

```bash
# From your worktree
python ../scripts/wex-cli.py init
```

### "Path already reserved by another worktree"

```bash
# Check who has it reserved
python ../scripts/wex-cli.py check-collisions

# Coordinate with that worktree to release or share the path
```

### "Config validation failed"

```bash
python config/validate.py --verbose
# Fix the reported errors
```

### "Harness drift detected"

```bash
python scripts/check_harness_drift.py
# Compare output with AGENTS.md
# Update configs to match
```

---

## 📚 References

- [AGENTS.md](../AGENTS.md) - Harness rules and constraints
- [TEAM_COLLABORATION.md](../TEAM_COLLABORATION.md) - Team workflow guide
- [CONFIG_SETUP.md](../CONFIG_SETUP.md) - Configuration quick start
- [docs/architecture/CONFIG_ARCHITECTURE.md](../docs/architecture/CONFIG_ARCHITECTURE.md) - Config architecture
- [WEX Protocol](docs/architecture/WEX_PROTOCOL.md) - Worktree Exchange Protocol

---

## ✅ Onboarding Checklist

- [ ] Cline fabric verified (`ls .cline/`)
- [ ] WEX initialized in my worktree
- [ ] Heartbeat emitted with my task
- [ ] Dashboard shows my worktree as active
- [ ] Path reservations made for my work
- [ ] No collisions detected
- [ ] Config validation passes
- [ ] Harness drift check passes
- [ ] I know which agent to ask for each type of work
- [ ] I know the commit message conventions
- [ ] I know the trust boundaries and forbidden actions

**You're ready!** 🎉
