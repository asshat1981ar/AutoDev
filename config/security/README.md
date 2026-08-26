# AutoDev Security Configuration

This directory contains security scanning and linting configurations for AutoDev.

## Structure

```
config/security/
├── README.md                    # This file
├── ast-grep.yml                 # Ast-grep rule definitions
├── ast-grep/                    # Additional ast-grep configs
│   └── rules/                   # Custom rules directory
├── coderabbit-instructions.md   # CodeRabbit review instructions
└── semgrep/                     # Semgrep rules (if used)
```

## Security Boundaries

AutoDev enforces strict trust boundaries:

1. **ForgeCore** (`crates/forge-core/`) - Trusted execution kernel
2. **Control Plane** (`crates/autodev-server/`) - Untrusted adapter
3. **Kotlin Modules** (`kotlin/`) - Control plane with commonMain purity
4. **Python Fabric** (`.cline/`, `scripts/`) - Cline development fabric
5. **Observability** (`scripts/autodev-cli.py`, `web/`) - Read-only

## Enforcement Tools

### Ast-grep

Ast-grep is used for pattern-based security checks. Rules are defined in:
- `config/security/ast-grep.yml` - Main rule set
- `config/security/ast-grep/rules/` - Custom rules
- `.ast-grep/rules/` - Existing repository rules (preserved)

### CodeRabbit

CodeRabbit provides AI-assisted code review with path-specific instructions:
- `.coderabbit.yaml` - Main configuration
- `config/security/coderabbit-instructions.md` - Extended instructions

## Rule Categories

### 1. Trust Boundary Violations

Prevent patterns that bypass authorization or execution boundaries:
- Direct process execution without Workspace confinement
- AuthorizationGrant creation outside ForgeCore
- Secret access in untrusted contexts

### 2. Authority Escalation

Prevent patterns that escalate privileges:
- Verification results treated as authorization
- Self-approval of effects
- Bypassing policy checks

### 3. Information Disclosure

Prevent patterns that leak sensitive information:
- Logging secrets or credentials
- Returning internal state to untrusted callers
- Cross-workspace information exposure

### 4. Code Quality & Determinism

Prevent patterns that violate determinism requirements:
- Randomness without seeding in tests
- Time-based assertions without mocking
- Network calls in deterministic tests

## Usage

### Ast-grep Scanning

```bash
# Scan entire repository
ast-grep scan --config config/security/ast-grep.yml .

# Scan specific component
ast-grep scan --config config/security/ast-grep.yml crates/forge-core/

# List all rules
ast-grep rules --config config/security/ast-grep.yml
```

### In CI

Ast-grep rules are validated as part of the harness drift check:
```bash
python scripts/check_harness_drift.py --verbose
```

## Adding New Rules

1. Create the rule file in `config/security/ast-grep/rules/`
2. Add it to `config/security/ast-grep.yml`
3. Document it in this README
4. Add corresponding tests if applicable
5. Ensure it doesn't conflict with existing rules

## Rule Priorities

- **Error** - Must be fixed before merge (blocks CI)
- **Warning** - Should be fixed ( CI warning)
- **Info** - Informational (CI pass but reported)

## False Positives

To handle false positives:

1. Add an exception comment: `// ast-grep: ignore <rule-id>`
2. Document the exception in the rule file
3. Consider narrowing the rule pattern

## Integration with Existing Rules

The `.ast-grep/rules/` directory contains existing repository-specific rules:
- `forgecore-no-unwrap.yml` - Prevents unwrap in ForgeCore
- `observer-no-process-exec.yml` - Prevents process exec in observer
- `python-no-shell-true.yml` - Prevents shell=True in Python
- `rust-no-direct-process-command.yml` - Prevents direct process commands

These are preserved and referenced in the centralized configuration.
