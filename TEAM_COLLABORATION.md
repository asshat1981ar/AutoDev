# AutoDev Team Collaboration Guide

## Overview

This guide describes how teams can collaborate effectively using AutoDev's Cline development fabric, centralized configuration architecture, and WEX (Worktree Exchange Protocol) for multi-agent coordination.

## Team Structure

AutoDev uses a **role-based agent system** with specialized capabilities:

### Agent Roles

| Role | Capability | Responsibility | Tools |
|------|------------|----------------|-------|
| **Config Manager** | S1 (Plan Mode) | Configuration custodianship | `config/`, Cline tasks |
| **Architect** | S1 | Design & architecture | ADR, PLANS.md |
| **Builder** | S0 | Implementation | cargo, gradle, python |
| **Reviewer** | S1 | Code review | CodeRabbit, ast-grep |
| **Security** | S1 | Security boundaries | ast-grep, policies |
| **Verifier** | S0 | Verification | CI, tests |
| **Scout** | S0 | Reconnaissance | grep, find |

### Team Composition

**Small Team (2-3 developers)**:
- 1x Config Manager (also does Architect)
- 1x Builder (also does Verifier)
- 1x Security/Reviewer (rotating)

**Medium Team (4-6 developers)**:
- 1x Config Manager
- 1x Architect
- 2x Builders
- 1x Security
- 1x Reviewer

**Large Team (7+ developers)**:
- 1x Config Manager
- 1x Architect
- 3-4x Builders
- 1x Security
- 1-2x Reviewers
- Optional: Dedicated Verifier

## Collaboration Workflows

### 1. Configuration Management Workflow

**Trigger**: Need to update or audit configurations

```
User Request → Config Manager → Pre-Config-Change Hook → Builder/Verifier → Commit
                      ↓
               Validation (config/validate.py)
                      ↓
               Harness Drift Check
                      ↓
               Evidence Push (WEX)
```

**Example Session**:
```
User: "Update Kotlin version to 2.0.22"

Config Manager:
1. Validates current state with config/validate.py
2. Identifies impacted files: config/kotlin/gradle.properties
3. Delegates to Builder with instructions

Builder:
1. Updates config/kotlin/gradle.properties
2. Runs config/validate.py
3. Runs scripts/check_harness_drift.py
4. Commits with message: "chore(config): Update Kotlin to 2.0.22"

Config Manager:
1. Pushes evidence to WEX registry
2. Updates heartbeat with new milestone
```

### 2. Feature Development Workflow

**Trigger**: New feature or enhancement

```
User Request → Architect → Config Manager → Builders → Security → Reviewer → Verifier
                      ↓                          ↓                ↓              ↓
               ADR/Design              Configuration    Static         Code
               Document                    Changes          Analysis      Review
```

**Example**: Adding a new MCP integration
```
1. User: "Add support for new MCP server X"
2. Architect: Creates ADR-00N-mcp-server-x.md
3. Config Manager: Identifies config changes needed
4. Builders: Implement the feature
5. Security: Reviews for trust boundary violations
6. Reviewer: Code review with CodeRabbit
7. Verifier: Runs full test suite
8. Config Manager: Pushes evidence to WEX
```

### 3. Configuration Audit Workflow

**Trigger**: Scheduled or after major changes

```
User/Cron → Config Manager → Security → Verifier → Config Manager
                    ↓              ↓            ↓              ↓
             Run validator   Review rules   Run tests      Generate report
```

**Steps**:
1. Config Manager runs `config-audit` task
2. Security agent reviews security configurations
3. Verifier runs integration tests
4. Config Manager generates audit report
5. Evidence pushed to WEX registry

## Multi-Worktree Collaboration (WEX)

WEX (Worktree Exchange Protocol) enables multiple agents/developers to work in parallel worktrees without conflicts.

### Worktree Setup

```bash
# Initialize a new worktree
git worktree add -b feature/new-feature ../autodev-new-feature origin/main
cd ../autodev-new-feature

# Initialize WEX
python scripts/wex-cli.py init --purpose "Feature: new-feature"

# Start work
python scripts/wex-cli.py heartbeat --task "Implementing new feature" --status active
```

### Path Reservation System

Prevent multiple worktrees from modifying the same files:

```bash
# Worktree A reserves config/ directory
python scripts/wex-cli.py reserve config/

# Worktree B tries to reserve config/ - BLOCKED
python scripts/wex-cli.py reserve config/
# Error: BLOCKED: 'config/' reserved by worktree 'worktree-a'

# Worktree B can work on different paths
python scripts/wex-cli.py reserve scripts/
```

### Evidence Exchange

Share evidence between worktrees:

```bash
# Worktree A pushes evidence
python scripts/wex-cli.py push-evidence my-evidence.json

# Worktree B pulls evidence
python scripts/wex-cli.py pull-evidence --source worktree-a

# View all evidence in registry
python scripts/wex-cli.py pull-evidence
```

### Worktree Dashboard

Monitor all active worktrees:

```bash
python scripts/wex-cli.py dashboard
```

Output:
```
WORKTREE                  STATUS     TASK                      LAST SEEN
--------------------------------------------------------------------------------------
primary                   active     Team dev with Cline       2026-08-21T13:54:24
feature/new-feature        active     Implementing new feature  2026-08-21T13:45:00
fix/security-issue        blocked    Waiting for review        2026-08-21T13:30:00

=== Collision Check ===
No collisions detected.

=== Evidence Registry ===
  5 evidence records
```

## Communication Patterns

### 1. Direct Agent Communication

```
User → Config Manager: "What's the current Rust toolchain version?"
Config Manager: "Rust toolchain is 'stable' via dtolnay/rust-toolchain@stable"
```

### 2. Task Delegation

```
User → Architect: "We need to upgrade Gradle to 8.11"
Architect → Config Manager: "Update Gradle version, verify all dependencies"
Config Manager → Builder: "Update config/kotlin/gradle.properties and config/ci/github/env.yaml"
```

### 3. Parallel Work with Coordination

```
# Worktree A (Builder)
python scripts/wex-cli.py reserve crates/forge-core/
python scripts/wex-cli.py heartbeat --task "ForgeCore updates" --status active

# Worktree B (Config Manager)  
python scripts/wex-cli.py reserve config/
python scripts/wex-cli.py heartbeat --task "Config updates" --status active

# Both can work in parallel without conflicts
```

### 4. Evidence-Based Handoffs

```
# Worktree A completes work and pushes evidence
python scripts/wex-cli.py push-evidence forge-core-evidence.json

# Worktree B pulls evidence before starting dependent work
python scripts/wex-cli.py pull-evidence --source worktree-a
```

## Team Coordination Commands

### Daily Standup

```bash
# All team members update their heartbeat
python scripts/wex-cli.py heartbeat --task "[Today's task]" --status active

# Team lead checks dashboard
python scripts/wex-cli.py dashboard

# Check for any collisions
python scripts/wex-cli.py check-collisions
```

### Sprint Planning

```bash
# Architect creates plan
# Config Manager sets up configuration requirements
# Each developer reserves their work paths
python scripts/wex-cli.py reserve [path]

# Start sprint
python scripts/wex-cli.py heartbeat --task "Sprint [N] - [Epic]" --status active
```

### Code Review Coordination

```bash
# Developer pushes code and evidence
python scripts/wex-cli.py push-evidence pr-evidence.json

# Reviewer pulls evidence
python scripts/wex-cli.py pull-evidence --source [worktree]

# Reviewer runs checks
python scripts/check_harness_drift.py
python config/validate.py
```

## Configuration-Specific Collaboration

### Making Configuration Changes

**Always use the config-update task**:

```bash
# 1. Check current state
python config/validate.py
python scripts/check_harness_drift.py

# 2. Make changes (through Cline or manually)
#    - Edit configuration files
#    - Maintain formatting (100 char, 2 spaces)
#    - Update version pins consistently

# 3. Validate changes
python config/validate.py --strict
python scripts/check_harness_drift.py

# 4. Test changes
# Rust
cd crates && cargo check && cd ..

# Kotlin
cd kotlin && ./gradlew --dry-run && cd ..

# Python
python -m py_compile config/validate.py
python -m unittest discover -s tests -v

# 5. Commit with proper message
# Use prefixes: feat(config), fix(config), chore(config), docs(config)

# 6. Push evidence to WEX
python scripts/wex-cli.py push-evidence evidence.json

# 7. Update heartbeat
python scripts/wex-cli.py heartbeat --task "Configuration update" --status completed
```

### Configuration Audit Schedule

| Frequency | Trigger | Agent | Task |
|-----------|---------|-------|------|
| Daily | Start of day | Config Manager | Quick validation |
| Weekly | Monday | Config Manager | Full audit |
| Before PR | Pre-merge | Reviewer | Config review |
| Before Release | Pre-release | Security | Security audit |
| After Incident | Post-mortem | All | Full audit + evidence review |

## Cline Task Catalog

| Task ID | Description | Duration | Agents |
|---------|-------------|----------|--------|
| `config-audit` | Comprehensive configuration audit | 30-60 min | Config Manager, Security, Verifier |
| `config-update` | Update specific configuration | 15-45 min | Config Manager, Builder |
| `harness-verify` | Full harness verification | 20-40 min | Config Manager, Verifier |
| `security-review` | Security-focused review | 25-50 min | Security, Reviewer |

## Best Practices

### 1. Always Reserve Paths

```bash
# Before starting work on any configuration file
python scripts/wex-cli.py reserve config/kotlin/

# After completing work
# (reservation is automatically cleared on new reservations)
```

### 2. Use Cline Tasks for Configuration Work

Instead of manual changes, use the structured tasks:
- `config-audit` for comprehensive reviews
- `config-update` for controlled changes

### 3. Maintain Evidence Trail

```bash
# Before any significant change
python scripts/wex-cli.py push-evidence before-change-evidence.json

# After completing the change
python scripts/wex-cli.py push-evidence after-change-evidence.json
```

### 4. Communicate via Heartbeat

```bash
# When starting work
python scripts/wex-cli.py heartbeat --task "Task description" --status active

# When blocked
python scripts/wex-cli.py heartbeat --task "Task description" --status blocked

# When completing
python scripts/wex-cli.py heartbeat --task "Task description" --status completed --milestone "Milestone"
```

### 5. Respect Trust Boundaries

- **ForgeCore**: Only Config Manager and Security can make changes
- **Control Plane**: Config Manager must review all changes
- **Kotlin commonMain**: Security must verify no platform types
- **Python scripts**: Must remain stdlib-only

## Emergency Procedures

### Configuration Drift Detected

```bash
# 1. Run drift check
python scripts/check_harness_drift.py --verbose

# 2. Identify the drift
# Check output for specific failures

# 3. Fix the drift
# Update either documentation or code to match

# 4. Push evidence
python scripts/wex-cli.py push-evidence drift-fix-evidence.json

# 5. Notify team
python scripts/wex-cli.py heartbeat --status blocked --milestone "Drift detected - fixing"
```

### Merge Conflict in Configuration

```bash
# 1. Check reservations
python scripts/wex-cli.py list
python scripts/wex-cli.py check-collisions

# 2. Identify conflicting worktrees
python scripts/wex-cli.py pull-evidence --source [worktree]

# 3. Coordinate resolution
# Manual coordination between worktree owners

# 4. Resolve and verify
python config/validate.py
python scripts/check_harness_drift.py
```

### Security Issue in Configuration

```bash
# 1. Immediately block affected worktrees
python scripts/wex-cli.py heartbeat --status blocked --task "Security issue"

# 2. Engage Security agent
# Security agent reviews and remediates

# 3. Push security evidence
python scripts/wex-cli.py push-evidence security-incident-evidence.json

# 4. Resume work after fix
python scripts/wex-cli.py heartbeat --status active
```

## Metrics and Reporting

### Team Velocity

```bash
# Count completed evidence in last week
python scripts/wex-cli.py pull-evidence | grep "passed" | wc -l

# List all completed work
python scripts/wex-cli.py pull-evidence
```

### Configuration Quality

```bash
# Check validation history
# (Track in git history of config/validate.py results)

# Check drift history
# (Track in git history of check_harness_drift.py results)
```

### Agent Performance

```bash
# Count evidence by source worktree
python scripts/wex-cli.py pull-evidence
# (Group by _wex_source field)
```

## Getting Help

### Ask Config Manager

```
"Config Manager, what's the current Rust toolchain version?"
"Config Manager, how do I add a new configuration file?"
"Config Manager, validate my changes"
```

### Ask Architect

```
"Architect, should we upgrade Gradle to 8.11?"
"Architect, what's the design for the new feature?"
"Architect, review this ADR"
```

### Ask Security

```
"Security, is this configuration change safe?"
"Security, review this ast-grep rule"
"Security, what are the trust boundary implications?"
```

### Ask Verifier

```
"Verifier, do my changes break anything?"
"Verifier, run the full test suite"
"Verifier, validate this configuration"
```

## Quick Reference Card

```
┌─────────────────────────────────────────────────────────────────┐
│                    TEAM COLLABORATION QUICK REF                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                     │
│  WORKFLOW:                                                          │
│    Plan → Reserve → Work → Validate → Push Evidence → Update HB    │
│                                                                     │
│  COMMANDS:                                                         │
│    wex-cli.py init                    # Initialize worktree        │
│    wex-cli.py heartbeat --task X     # Update status              │
│    wex-cli.py reserve path          # Reserve file/directory     │
│    wex-cli.py check-collisions       # Check for conflicts        │
│    wex-cli.py dashboard              # View all worktrees         │
│    wex-cli.py push-evidence file    # Share evidence              │
│    wex-cli.py pull-evidence          # View evidence              │
│                                                                     │
│  CONFIG COMMANDS:                                                   │
│    config/validate.py               # Validate all configs        │
│    config/validate.py --strict     # Strict validation           │
│    scripts/check_harness_drift.py   # Harness consistency         │
│                                                                     │
│  AGENTS:                                                           │
│    Config Manager - Configuration expertise                        │
│    Architect - Design & architecture                               │
│    Builder - Implementation                                          │
│    Security - Security review                                       │
│    Reviewer - Code review                                            │
│    Verifier - Verification & testing                                │
│                                                                     │
└─────────────────────────────────────────────────────────────────┘
```

## Resources

- [AGENTS.md](AGENTS.md) - Repository harness rules
- [PLANS.md](PLANS.md) - ExecPlan coordination
- [CONFIG_SETUP.md](CONFIG_SETUP.md) - Configuration setup guide
- [config/README.md](config/README.md) - Configuration documentation
- [.cline/README.md](.cline/README.md) - Cline development fabric
- [.cline/agents/config-manager.md](.cline/agents/config-manager.md) - Config Manager instructions
- [scripts/wex-cli.py --help](scripts/wex-cli.py) - WEX CLI help

## Checklist for New Team Members

- [ ] Read AGENTS.md and understand harness rules
- [ ] Read CONFIG_SETUP.md and understand configuration architecture
- [ ] Read this TEAM_COLLABORATION.md guide
- [ ] Initialize WEX in your worktree
- [ ] Reserve your initial work paths
- [ ] Update your heartbeat with current task
- [ ] Join the appropriate agent channel
- [ ] Run config/validate.py to verify your environment

## Glossary

| Term | Definition |
|------|------------|
| WEX | Worktree Exchange Protocol - Coordination system for multi-agent development |
| Worktree | Separate git worktree for parallel development |
| Evidence | JSON record of completed work or verification |
| Registry | Shared directory for evidence exchange |
| Heartbeat | Status update from a worktree |
| Reservation | Advisory lock on a file/directory path |
| S0/S1/S2 | Capability levels (S0=Direct, S1=Plan Mode, S2=Deep Planning) |
