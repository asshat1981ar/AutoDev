# Task: Configuration Audit

**ID**: config-audit
**Type**: Maintenance / Quality Assurance
**Priority**: Medium
**Estimated Duration**: 30-60 minutes
**Required Agents**: Config Manager, Security, Verifier

## Objective

Perform a comprehensive audit of all AutoDev configuration files to ensure:
1. All configurations respect AGENTS.md harness rules
2. Version pins are consistent across the repository
3. All configuration files have valid syntax
4. No forbidden root manifests exist
5. Security rules are properly enforced

## Prerequisites

- Python 3.10 or 3.11 installed
- Cline environment set up
- Repository cloned and up-to-date

## Steps

### Step 1: Run Configuration Validator

**Agent**: Config Manager
**Action**: Execute configuration validation

```bash
cd /path/to/AutoDev
python config/validate.py --verbose
```

**Acceptance Criteria**:
- [ ] 0 errors reported
- [ ] All warnings reviewed and addressed or documented

**Artifacts**:
- Validation report (stdout)

### Step 2: Run Harness Drift Check

**Agent**: Config Manager
**Action**: Verify harness consistency

```bash
python scripts/check_harness_drift.py --verbose
```

**Acceptance Criteria**:
- [ ] "Harness drift check: PASS" message
- [ ] No drift detected between documentation and CI

**Artifacts**:
- Drift check report

### Step 3: Verify Version Pins

**Agent**: Config Manager
**Action**: Cross-check all version pins

**Checklist**:
- [ ] Rust toolchain: stable (matches dtolnay/rust-toolchain@stable)
- [ ] Gradle: 8.10.2 (matches AGENTS.md and config/ci/github/env.yaml)
- [ ] Kotlin: 2.0.21 (matches config/kotlin/gradle.properties)
- [ ] ktlint: 12.1.1 (matches config/kotlin/gradle.properties)
- [ ] Node: 24 (matches AGENTS.md)
- [ ] Python: 3.10, 3.11 (matches AGENTS.md)
- [ ] JDK: 17 (matches AGENTS.md)
- [ ] Android SDK: 35 (matches AGENTS.md)
- [ ] Android Build Tools: 35.0.0 (matches AGENTS.md)

**Command**:
```bash
grep -r "gradle\|kotlin\|jdk\|node\|python" config/ AGENTS.md .github/workflows/ci.yml
```

### Step 4: Security Configuration Review

**Agent**: Security
**Action**: Review security-sensitive configurations

**Checklist**:
- [ ] `config/security/ast-grep.yml` rules are comprehensive
- [ ] All trust boundary rules are present
- [ ] No secrets or credentials in configuration files
- [ ] `config/local/` is properly gitignored

**Files to Review**:
- `config/security/ast-grep.yml`
- `.coderabbit.yaml`
- `.cline/config/permissions.json`
- `.cline/config/capabilities.json`

### Step 5: Test Configuration Integration

**Agent**: Verifier
**Action**: Verify configurations work with actual builds

**Test Commands**:
```bash
# Rust - from crates/ directory
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cd ..

# Kotlin - from kotlin/ directory  
cd kotlin
./gradlew ktlintCheck --no-daemon
cd ..

# Python - from repo root
python -m py_compile config/validate.py config/*.md
python -m unittest discover -s tests -v

# Node - from repo root
node --check scripts/termux-kanban.mjs
```

**Acceptance Criteria**:
- [ ] All format checks pass
- [ ] All lint checks pass
- [ ] All compilation checks pass
- [ ] All tests pass

### Step 6: Documentation Review

**Agent**: Config Manager
**Action**: Ensure all configurations are documented

**Checklist**:
- [ ] `config/README.md` is up-to-date
- [ ] `CONFIG_SETUP.md` is accurate
- [ ] All subdirectory README files exist (optional but recommended)
- [ ] Version pins are documented

### Step 7: Create Audit Report

**Agent**: Config Manager
**Action**: Generate audit report

**Template**:
```markdown
# Configuration Audit Report

**Date**: YYYY-MM-DD
**Auditor**: [Agent/AutoDev]
**Status**: PASS/FAIL

## Summary

[Brief summary of findings]

## Findings

### Issues Found
- [ ] Issue 1: [Description]
- [ ] Issue 2: [Description]

### Warnings
- [ ] Warning 1: [Description]

### Recommendations
- [ ] Recommendation 1: [Description]

## Version Consistency

| Component | Expected | Actual | Status |
|-----------|----------|--------|--------|
| Rust Toolchain | stable | [value] | ✅/❌ |
| Gradle | 8.10.2 | [value] | ✅/❌ |
| ... | ... | ... | ... |

## Check Results

- Configuration Validator: [PASS/FAIL]
- Harness Drift Check: [PASS/FAIL]
- Security Review: [PASS/FAIL]
- Integration Tests: [PASS/FAIL]

## Action Items

1. [ ] [Action item 1]
2. [ ] [Action item 2]
```

## Success Criteria

All of the following must be true:
1. ✅ Configuration validator reports 0 errors
2. ✅ Harness drift check passes
3. ✅ All version pins are consistent
4. ✅ Security configurations are valid
5. ✅ Integration tests pass
6. ✅ Documentation is up-to-date

## Scheduling

**Frequency**: Run this task:
- After any configuration change
- Weekly as part of maintenance
- Before any major release
- When updating tool versions

**Automation**: This task can be partially automated by running:
```bash
python config/validate.py && python scripts/check_harness_drift.py
```

## Related Tasks

- `config-update` - Update specific configuration
- `config-review` - Review configuration PR
- `harness-verify` - Full harness verification

## Escalation

If any critical issues are found:
1. Stop the release process
2. Engage Security agent for security issues
3. Engage Architect for architectural issues
4. Create issues for each finding
5. Block merging until resolved
