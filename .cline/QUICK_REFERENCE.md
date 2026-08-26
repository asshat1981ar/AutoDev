# Cline Quick Reference Card

## 🎯 Daily Commands

```bash
# Validate everything
python config/validate.py && python scripts/check_harness_drift.py

# Check team status
python scripts/wex-cli.py dashboard

# Check for path conflicts
python scripts/wex-cli.py check-collisions

# Reserve a path
python scripts/wex-cli.py reserve <path>

# List worktrees
python scripts/wex-cli.py list
```

---

## 🤖 Agent Quick Reference

| When you need... | Ask this agent |
|-----------------|----------------|
| Config questions | `Config Manager` |
| Architecture advice | `Architect` |
| Implement code | `Builder` |
| Code review | `Reviewer` |
| Testing/verification | `Verifier` |
| Security check | `Security` |
| Explore repo | `Scout` |

---

## 📋 Common Workflows

### Start a development cycle
```
Task: team-development-cycle cycle_name="Kotlin Integration" worktrees=["primary","team-1"]
```

### Update configuration
```
Task: config-update component=kotlin file=gradle.properties change="update ktlint" reason="ADR-001"
```

### Audit configurations
```
Task: config-audit
```

---

## 🔒 Trust Boundaries (DO NOT VIOLATE)

### ❌ FORBIDDEN
- Root `Cargo.toml`, `package.json`, `pyproject.toml` without ADR
- `AuthorizationGrant` outside ForgeCore
- Direct process execution bypassing Workspace
- Secrets in config files
- Fail-open public adapters

### ✅ REQUIRED
- Configs match AGENTS.md
- Versions match CI workflow
- Kotlin `commonMain` purity
- CLI read-only observer

---

## 📁 File Locations

```
Repository Root
├── .cline/                      # Cline fabric
│   ├── agents/                  # Agent definitions
│   ├── tasks/                   # Structured tasks
│   ├── hooks/                   # Pre/post hooks
│   └── config/                  # Cline config
│
├── config/                      # Centralized config
│   ├── defaults/                # Shared defaults
│   ├── rust/                    # Rust config
│   ├── kotlin/                  # Kotlin config
│   ├── python/                  # Python config
│   ├── security/                # Security config
│   └── ci/                      # CI config
│
├── scripts/                     # Scripts
│   ├── wex-cli.py               # WEX protocol CLI
│   ├── check_harness_drift.py   # Harness validation
│   └── ...
│
└── .worktrees/                  # Git worktrees
    ├── team-1/                  # Team worktree
    │   └── .wex/                # WEX state
    └── .registry/               # Shared registry
        ├── manifest.json        # Worktree identities
        ├── reservations.json    # Path reservations
        └── evidence-exchange/    # Evidence records
```

---

## 🎨 Commit Message Conventions

```
feat(kotlin): Add MPP core module
fix(config): Update Gradle properties
chore(ci): Add GitHub Actions env
docs: Update CONFIG_SETUP.md
feat(forge-core): Add trust boundary checks
```

---

## 🚨 Emergency Commands

```bash
# Restore WEX from git
git show 1d56331:scripts/wex-cli.py > scripts/wex-cli.py

# Check what's broken
python config/validate.py --verbose
python scripts/check_harness_drift.py

# See all worktrees
git worktree list

# Remove a worktree (careful!)
git worktree remove .worktrees/<name>
git branch -D <branch>
```

---

## ✅ Pre-Commit Checklist

- [ ] `python config/validate.py` passes
- [ ] `python scripts/check_harness_drift.py` passes
- [ ] `python scripts/wex-cli.py check-collisions` shows no collisions
- [ ] Paths I modified are reserved for my worktree
- [ ] Commit message follows conventions
- [ ] Evidence generated for significant changes

---

## 📊 WEX Dashboard Example

```
WORKTREE     STATUS   TASK                          LAST SEEN              BRANCH
----------------------------------------------------------------------------------------------
primary      active   Team development              2026-08-21T14:19:00  feat/coderabbit-config
team-1       active   Kotlin MPP implementation    2026-08-21T14:08:00  feat/team-dev-1

=== Collision Check ===
No collisions detected.

=== Evidence Registry ===
4 evidence records
```

---

## 🔗 Useful Links

- **[AGENTS.md](../AGENTS.md)** - Harness rules
- **[TEAM_COLLABORATION.md](../TEAM_COLLABORATION.md)** - Team guide
- **[CONFIG_SETUP.md](../CONFIG_SETUP.md)** - Config quick start
- **[Cline Onboarding](./CLINE_ONBOARDING.md)** - Full onboarding
- **[WEX Protocol](../docs/architecture/WEX_PROTOCOL.md)** - Worktree Exchange
