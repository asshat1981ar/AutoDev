# AutoDev Configuration Architecture

## Overview

AutoDev uses a hierarchical, component-specific configuration model that respects the repository's ownership boundaries and harness enforcement rules. This document describes the ideal configuration setup that balances centralization with component autonomy.

## Configuration Hierarchy

```
config/                      # Root configuration directory (new)
├── README.md               # This document
├── defaults/               # Shared default configurations
│   ├── clippy.toml         # Rust clippy lint configuration
│   ├── rustfmt.toml        # Rust formatting configuration
│   └── common/             # Cross-language shared configs
│       ├── editorconfig    # Editor configuration
│       └── prettier/       # Formatting (if needed)
├── rust/                   # Rust workspace configuration
│   ├── toolchain.toml      # Rust toolchain pins
│   └── features/           # Feature flag configurations
├── kotlin/                 # Kotlin workspace configuration  
│   ├── gradle.properties   # Shared Gradle properties
│   └── ktlint/             # Ktlint configuration
├── python/                 # Python configuration
│   ├── mypy.ini            # Type checking
│   └── pytest.ini          # Test configuration
├── security/               # Security scanning configurations
│   ├── ast-grep.yml        # Ast-grep rules
│   └── semgrep/            # Semgrep rules
└── ci/                    # CI-specific overrides
    ├── github/             # GitHub Actions specific
    └── local/              # Local development overrides
```

## Design Principles

### 1. **No Root Manifests Without ADR**
Per AGENTS.md section 4, we do NOT create:
- Root `Cargo.toml`
- Root `package.json`
- Root `pyproject.toml`

All configuration lives in component-specific or `config/` directories.

### 2. **Harness-Enforced Consistency**
Configuration must be:
- Checked by `scripts/check_harness_drift.py`
- Validated against `.github/workflows/ci.yml`
- Reproducible across environments

### 3. **Component Ownership**
Each language/component owns its configuration:
- Rust: `crates/` + `config/rust/`
- Kotlin: `kotlin/` + `config/kotlin/`
- Python: `config/python/` + `.cline/`
- CI: `.github/workflows/` + `config/ci/`

### 4. **Environment Separation**
- **Default configs**: Committed in `config/defaults/`
- **Local overrides**: `.gitignore`d, use `config/local/` (not committed)
- **CI overrides**: Environment variables or `config/ci/`

## Component Configurations

### Rust Configuration

**Location**: `config/rust/` and `crates/Cargo.toml`

#### Shared Toolchain
```toml
# config/rust/toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

#### Clippy Configuration
```toml
# config/rust/clippy.toml
[checks]
# Deny warnings in trusted boundary
unnecessary_wraps = "deny"
used_underscore_bindings = "deny"

[overrides]
# Allow in test code
[[checks.overrides]]
files = ["tests/**"]
unnecessary_wraps = "warn"
```

#### Rustfmt Configuration
```toml
# config/rust/rustfmt.toml
edition = "2021"
max_width = 100
tab_spaces = 2
newline_style = "Unix"
```

### Kotlin Configuration

**Location**: `config/kotlin/` and `kotlin/*.gradle.kts`

#### Shared Gradle Properties
```properties
# config/kotlin/gradle.properties
# Version pins
kotlin.version=2.0.21
gradle.version=8.10.2
ktlint.version=12.1.1

# JVM toolchain
org.gradle.java.home=$JDK_17_HOME
```

#### Ktlint Configuration
```yaml
# config/kotlin/ktlint/.ktlint.yaml
editorconfig:
  indent_size: 2
  max_line_length: 100

rules:
  no-wildcard-imports: true
  indent:
    auto-correction: true
```

### Python Configuration

**Location**: `config/python/`

#### Mypy Configuration
```ini
# config/python/mypy.ini
[mypy]
python_version = 3.10
warn_return_any = True
warn_unused_ignores = True
ignore_missing_imports = True
```

#### Pytest Configuration
```ini
# config/python/pytest.ini
[pytest]
testpaths = tests
python_files = test_*.py
addopts = -v --tb=short
```

## Security Configuration

### Ast-grep Rules
```yaml
# config/security/ast-grep.yml
rules:
  - id: rust-no-direct-process-command
    pattern: std::process::Command
    message: "Direct process execution bypasses Workspace confinement"
    severity: error
    languages: [rust]
    paths:
      include:
        - crates/forge-core/src/**
      exclude:
        - crates/forge-core/tests/**
```

## CI-Specific Configuration

### GitHub Actions Environment
```yaml
# config/ci/github/env.yaml
env:
  GRADLE_OPTS: -Dorg.gradle.daemon=false
  CARGO_TERM_COLOR: always
  AUTODEV_PORT: 8080
```

## Validation Rules

1. **Drift Check**: All config files must have corresponding entries in `ci.yml`
2. **No Duplication**: Component configs must not duplicate workspace-level settings
3. **Deterministic**: Config files must produce consistent behavior across runs
4. **Version Pins**: All tool versions must be explicitly pinned

## Integration with Existing System

The configuration architecture integrates with:
- `scripts/check_harness_drift.py` - Validates config consistency
- `.github/workflows/ci.yml` - Uses config for build/test commands
- `AGENTS.md` - Config constraints documented
- `PLANS.md` - Config changes tracked as ExecPlan milestones

## Migration Path

1. Create `config/` directory with defaults
2. Move existing scattered configs into `config/`
3. Update CI to reference `config/` files
4. Update drift check to validate configs
5. Document in this file
