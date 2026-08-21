# Development Cycle Plan: Kotlin MPP Integration with Centralized Config

**Cycle ID**: cycle-2026-08-21-kotlin-mpp
**Start Date**: 2026-08-21
**Status**: PLANNED → READY
**Worktrees**: primary, team-1
**Duration**: 5-7 days

---

## 🎯 Objectives

### Primary Objective
Integrate centralized configuration architecture (`config/`) with Kotlin Multiplatform Project (MPP) modules, establishing a reproducible pattern for all AutoDev Kotlin development.

### Secondary Objectives
1. Validate Cline fabric with real development workflow
2. Test WEX protocol for parallel team development
3. Document the integration pattern for future modules
4. Generate evidence for all development steps

---

## 📋 Scope

### In Scope
- [ ] Kotlin MPP module configuration integration
- [ ] `config/kotlin/gradle.properties` usage in all Kotlin modules
- [ ] `config/kotlin/ktlint/.ktlint.yaml` enforcement
- [ ] Cline agent delegation workflow
- [ ] WEX path reservation and collision prevention
- [ ] Evidence generation for all changes

### Out of Scope
- Rust module configuration (future cycle)
- Python fabric refactoring (future cycle)
- CI pipeline changes (separate task)
- Production deployment

---

## 👥 Team & Worktree Assignments

### Worktree: primary
- **Lead**: Primary development coordination
- **Branch**: `feat/engram-mcp-server` → `feat/cycle-kotlin-mpp`
- **Focus**: Infrastructure, cross-cutting concerns
- **Reserved Paths**: `crates/forge-core/src`, `.cline`, `scripts`, `skills`, `.vibe`
- **Agents**: Architect, Config Manager, Reviewer

### Worktree: team-1
- **Lead**: Kotlin MPP implementation
- **Branch**: `feat/team-dev-1`
- **Focus**: Kotlin module development
- **Reserved Paths**: `kotlin/mpp-core`
- **Agents**: Builder, Verifier, Config Manager

---

## 📅 Milestones

### Milestone 1: Setup & Planning (Day 0 - COMPLETE ✅)
- [x] Development cycle plan created
- [x] Worktrees configured (primary + team-1)
- [x] WEX protocol initialized
- [x] Cline onboarding documentation complete
- [x] Path reservations in place
- [x] Baseline evidence generated

**Evidence**: 
- `baseline-assessment-2026-08-21`
- `config-cline-integration-2026-08-21`
- `pr-organization-evidence-2026-08-21`

### Milestone 2: Infrastructure & Foundation (Day 0-1 - COMPLETE ✅)
- [x] 6 PR branches organized (wex, ci, vibe, engram, superpowers, amcx1)
- [x] Engram MCP server code fixed and tested (5/5 tests)
- [x] Vibe skills restored (amx-ecm, meta-creator, vibe-setup)
- [x] Superpower meta-skills created (subagent, cross-prompt, adversarial, reflexion)
- [x] templates.py quoting fix merged, templates_new.py removed
- [x] Engram onboarding memory stored
- [x] Vibe collaboration evidence pushed

**Evidence**: 
- `pr-organization-2026-08-21`
- `vibe-collaboration-2026-08-21`

### Milestone 3: MPP Core Integration (Day 1-2 — CURRENT)
**Target**: 2026-08-22 to 2026-08-23
**Reconciliation (2026-08-21, cycle closeout):** team-1 wired `kotlin/gradle.properties`
(copied from centralized config) and `kotlin/.ktlint.yaml` (symlink to
`config/kotlin/ktlint/.ktlint.yaml`). Build was recorded as "blocked: no JDK" —
**false blocker**: JDK 17 Corretto is installed at
`/opt/amazon-corretto-17.0.20.10.1-linux-aarch64`, just not on PATH. The copied
properties also inherited invalid lines (`$JDK_17_HOME`, Groovy block) — fixed at
source in 74765df; propagation to `kotlin/` pending.

- [x] Update `mpp-core/build.gradle.kts` to reference `config/kotlin/gradle.properties` *(via kotlin/gradle.properties copy — mechanism review pending)*
- [x] Verify ktlint uses `config/kotlin/ktlint/.ktlint.yaml` *(symlink in place — portability review pending)*
- [ ] Test build with `./gradlew :mpp-core:assemble --no-daemon`
- [ ] Run `python config/validate.py` from team-1
- [ ] Run `python scripts/check_harness_drift.py`
- [ ] Generate evidence: `team-1--mpp-core-integration-2026-08-22.json`

**Success Criteria**:
- ✅ MPP core builds successfully with centralized config
- ✅ All version pins match AGENTS.md
- ✅ No harness drift
- ✅ Evidence pushed to registry

**Worktree**: team-1
**Agents**: Builder, Verifier, Config Manager

### Milestone 4: MPP Server & UI Integration (Day 3-4)
**Target**: 2026-08-24 to 2026-08-25

- [ ] Apply same pattern to `mpp-server/build.gradle.kts`
- [ ] Apply same pattern to `mpp-ui/build.gradle.kts`
- [ ] Apply same pattern to `mpp-codegraph/build.gradle.kts`
- [ ] Run `./gradlew build --no-daemon` for all modules
- [ ] Run ktlint: `./gradlew ktlintCheck --no-daemon`
- [ ] Generate evidence for each module

**Success Criteria**:
- ✅ All Kotlin MPP modules build with centralized config
- ✅ ktlint passes with centralized rules
- ✅ All evidence records show "passed"

**Worktree**: team-1 (with assistance from primary)
**Agents**: Builder, Verifier, Config Manager, Reviewer

### Milestone 5: Documentation & Verification (Day 5)
**Target**: 2026-08-26

- [ ] Create `docs/architecture/KOTLIN_CONFIG_INTEGRATION.md`
- [ ] Document the pattern for future modules
- [ ] Run full verification suite
- [ ] Review with Security agent
- [ ] Generate completion evidence

**Success Criteria**:
- ✅ Documentation complete
- ✅ Security review passed
- ✅ All validation tests pass

**Worktree**: primary
**Agents**: Architect, Reviewer, Security, Config Manager

### Milestone 6: Merge & Cleanup (Day 6-7)
**Target**: 2026-08-27 to 2026-08-28

- [ ] Merge `feat/team-dev-1` into primary branch
- [ ] Resolve any merge conflicts
- [ ] Run final validation
- [ ] Clean up worktree (optional)
- [ ] Generate cycle completion evidence

**Success Criteria**:
- ✅ Code merged to main development branch
- ✅ All tests pass
- ✅ Evidence registry complete

**Worktree**: primary
**Agents**: Architect, Builder, Verifier

---

## 🎯 Success Criteria (Overall)

The development cycle is considered **SUCCESSFUL** when:

1. ✅ All Kotlin MPP modules use centralized configuration
2. ✅ All builds pass with `./gradlew build --no-daemon`
3. ✅ All ktlint checks pass with centralized rules
4. ✅ No harness drift introduced
5. ✅ No path collisions occurred
6. ✅ All changes pass config validation (0 errors)
7. ✅ Evidence generated for all significant work
8. ✅ Documentation updated
9. ✅ Security review completed
10. ✅ Code merged to primary branch

---

## 🛡️ Quality Gates

### Pre-Commit Gates (Must Pass)
```bash
# From any worktree
python ../config/validate.py
python ../scripts/check_harness_drift.py
python ../scripts/wex-cli.py check-collisions
```

### Pre-Merge Gates (Must Pass)
- [ ] All worktree changes validated
- [ ] No harness drift
- [ ] All evidence records show "passed"
- [ ] Documentation updated
- [ ] Security review completed

### Post-Merge Gates (Must Verify)
- [ ] CI pipeline passes (if configured)
- [ ] All worktrees still operational
- [ ] No new collisions introduced

---

## 📊 Monitoring & Reporting

### Daily Monitoring
```bash
# From primary worktree
python scripts/wex-cli.py dashboard
python config/validate.py
python scripts/check_harness_drift.py

# Check git status
git status --short
git worktree list
```

### Evidence Requirements
Each significant development step must generate an evidence record with:
- Evidence ID (unique, with date)
- Type (development, integration, verification)
- Result (passed, in_progress, failed)
- Description
- Worktree
- Branch
- Checks performed
- Files modified
- Config used

### Reporting
- **Daily**: Heartbeat update with milestone progress
- **Milestone**: Evidence record for each milestone completion
- **Cycle**: Final completion evidence with all metrics

---

## 🔗 Dependencies

### Dependencies Met
- ✅ Centralized config architecture (ffce47e)
- ✅ Cline integration (f1deaad)
- ✅ WEX protocol (1d56331)
- ✅ Team-1 worktree configured
- ✅ Cline onboarding documentation complete

### Blockers
None identified

### Risks
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Gradle version conflicts | Low | Medium | Use wrapper, verify versions |
| Kotlin version incompatibility | Low | Medium | Test with Gradle 8.10.2 + Kotlin 2.0.21 |
| Path reservation conflicts | Low | High | Check collisions before each commit |
| Build failures | Medium | High | Incremental commits, validate often |

---

## 📚 References

- [AGENTS.md](AGENTS.md) - Harness rules and constraints
- [TEAM_COLLABORATION.md](TEAM_COLLABORATION.md) - Team workflow guide
- [CONFIG_SETUP.md](CONFIG_SETUP.md) - Configuration quick start
- [.cline/CLINE_ONBOARDING.md](.cline/CLINE_ONBOARDING.md) - Cline onboarding
- [.cline/tasks/team-development-cycle.md](.cline/tasks/team-development-cycle.md) - Team dev workflow
- [docs/architecture/CONFIG_ARCHITECTURE.md](docs/architecture/CONFIG_ARCHITECTURE.md) - Config architecture
- [WEX Protocol](docs/architecture/WEX_PROTOCOL.md) - Worktree Exchange Protocol

---

## 🚀 Starting the Cycle

### From Primary Worktree
```bash
# Sync branches
git checkout feat/coderabbit-config
git pull origin feat/coderabbit-config

# Create cycle branch
git checkout -b feat/cycle-kotlin-mpp

# Update primary heartbeat
python scripts/wex-cli.py heartbeat \
  --task "Cycle 2026-08-21: Kotlin MPP Integration" \
  --status active \
  --milestone "Milestone 1: Setup Complete"
```

### From Team-1 Worktree
```bash
# Switch to cycle branch
git checkout feat/cycle-kotlin-mpp

# Refresh heartbeat
python ../scripts/wex-cli.py heartbeat \
  --task "Cycle 2026-08-21: MPP Core Integration" \
  --status active \
  --milestone "Milestone 2: Starting implementation"

# Verify reservations
python ../scripts/wex-cli.py check-collisions
```

### Start Development
```
# From team-1
Task: team-development-cycle cycle_name="Kotlin MPP Integration" worktrees=["primary","team-1"]
```

Or directly:
```
Builder, start MPP core integration with config/kotlin/gradle.properties
```

---

## 📝 Cycle Checklist

- [x] Cycle plan created
- [x] Worktrees assigned
- [x] Milestones defined
- [x] Success criteria established
- [x] Quality gates defined
- [x] Dependencies verified
- [x] Risks assessed
- [ ] Cycle branch created
- [ ] Heartbeats updated
- [ ] Development started

---

**Status**: ✅ READY TO START  
**Next Action**: Create cycle branch and begin Milestone 2  
**Plan Created**: 2026-08-21T14:35:00+00:00
