# AutoDev Configuration Setup

## Overview

This document describes the ideal configuration setup for AutoDev. The configuration follows a hierarchical, component-specific model that respects the repository's ownership boundaries and harness enforcement rules.

## Configuration Structure

```
config/                      # Centralized configuration directory
├── README.md               # Main configuration documentation
├── validate.py             # Configuration validator script
├── defaults/               # Shared default configurations
│   ├── clippy.toml         # Clippy lint recommendations
│   ├── rustfmt.toml        # Rust formatting defaults
│   └── common/             # Cross-language configs
│       └── .editorconfig    # Editor configuration
├── rust/                   # Rust workspace configuration
│   ├── toolchain.toml      # Rust toolchain specification
│   └── features/           # Feature flag configurations
│       └── forge-core.toml  # ForgeCore feature flags (future)
├── kotlin/                 # Kotlin workspace configuration
│   ├── gradle.properties   # Shared Gradle properties
│   └── ktlint/             # Ktlint configuration
│       └── .ktlint.yaml    # Ktlint rules
├── python/                 # Python configuration
│   ├── mypy.ini            # Static type checking
│   ├── pytest.ini          # Test framework (compatibility)
│   └── unittest.ini        # Unittest configuration (canonical)
├── security/               # Security scanning configurations
│   ├── README.md           # Security configuration documentation
│   └── ast-grep.yml        # Ast-grep security rules
└── ci/                    # CI-specific configurations
    ├── github/             # GitHub Actions
    │   └── env.yaml        # Environment variables
    └── local/              # Local development (gitignored)
        ├── .env.example     # Example local overrides
        └── .gitkeep         # Keep directory in git
```

## Design Principles

### 1. No Root Manifests Without ADR

Per **AGENTS.md section 4**, we do NOT create root-level package manifests:
- ❌ Root `Cargo.toml`
- ❌ Root `package.json`
- ❌ Root `pyproject.toml`

All configuration lives in `config/` or component-specific directories.

### 2. Harness-Enforced Consistency

All configurations are validated by:
- `scripts/check_harness_drift.py` - Ensures CI commands match documented commands
- `config/validate.py` - Validates configuration syntax and consistency
- `.github/workflows/ci.yml` - Uses configured tool versions

### 3. Component Ownership

Each language/component owns its configuration:

| Component | Configuration Location | Ownership |
|-----------|----------------------|-----------|
| Rust | `config/rust/`, `crates/Cargo.toml` | Rust team |
| Kotlin | `config/kotlin/`, `kotlin/*.gradle.kts` | Kotlin team |
| Python | `config/python/` | Python/Cline team |
| CI/CD | `.github/workflows/`, `config/ci/` | DevOps |
| Security | `config/security/` | Security |

### 4. Environment Separation

- **Default configs** (committed): `config/` directory
- **Local overrides** (gitignored): `config/local/`
- **CI overrides** (environment variables): `config/ci/github/env.yaml`

## Quick Start

### For Developers

#### 1. Use Centralized Configs

**Rust:**
```bash
# From crates/ directory
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

**Kotlin:**
```bash
# From kotlin/ directory
./gradlew clean test --no-daemon
./gradlew ktlintCheck --no-daemon
```

**Python:**
```bash
# From repo root
python -m py_compile scripts/ install.py .cline/hooks/*.py
python -m unittest discover -s tests -v
```

#### 2. Local Development Setup

```bash
# Copy the example environment file
cp config/ci/local/.env.example config/local/.env

# Edit with your local settings
nano config/local/.env

# Source it
source config/local/.env
```

### For CI Maintainers

The centralized configurations in `config/` are referenced by:
- `.github/workflows/ci.yml` - CI workflow definitions
- `scripts/check_harness_drift.py` - Drift detection
- `config/validate.py` - Configuration validation

## Configuration Files

### Rust Configuration

| File | Purpose | Usage |
|------|---------|-------|
| `config/rust/toolchain.toml` | Toolchain specification | Documentation only |
| `config/defaults/clippy.toml` | Lint recommendations | Documentation, inline attributes |
| `config/defaults/rustfmt.toml` | Formatting defaults | `cargo fmt -- --config-path` |

### Kotlin Configuration

| File | Purpose | Usage |
|------|---------|-------|
| `config/kotlin/gradle.properties` | Shared Gradle settings | Auto-loaded by Gradle |
| `config/kotlin/ktlint/.ktlint.yaml` | Ktlint rules | Gradle ktlint plugin |

### Python Configuration

| File | Purpose | Usage |
|------|---------|-------|
| `config/python/mypy.ini` | Type checking | `mypy --config-file` |
| `config/python/pytest.ini` | Test discovery | `pytest -c` |
| `config/python/unittest.ini` | Unittest settings | `unittest -c` |

### Security Configuration

| File | Purpose | Usage |
|------|---------|-------|
| `config/security/ast-grep.yml` | Security rules | `ast-grep scan --config` |
| `config/security/README.md` | Documentation | Reference |

### CI Configuration

| File | Purpose | Usage |
|------|---------|-------|
| `config/ci/github/env.yaml` | GitHub Actions env | Reference in workflows |
| `config/ci/local/.env.example` | Local template | Copy to `config/local/.env` |

## Validation

Run the configuration validator:

```bash
# Basic validation
python config/validate.py

# Verbose output
python config/validate.py --verbose

# Strict mode (warnings as errors)
python config/validate.py --strict

# Attempt auto-fixes (limited)
python config/validate.py --fix
```

The validator checks:
- ✅ Configuration directory structure
- ✅ No forbidden root manifests
- ✅ Tool version consistency with AGENTS.md
- ✅ Line length consistency across configs
- ✅ Symlink integrity
- ✅ Gitignore coverage
- ✅ Harness consistency
- ✅ File syntax validation

## Adding New Configuration

1. **Create the file** in the appropriate `config/<component>/` directory
2. **Document it** in the corresponding README or this file
3. **Update validation** in `config/validate.py` if needed
4. **Update CI** in `.github/workflows/ci.yml` if needed
5. **Verify** with `python config/validate.py`

## Version Pinning

All tool versions must be explicitly pinned:

| Tool | Version | Source |
|------|---------|--------|
| Rust toolchain | stable | dtolnay/rust-toolchain@stable |
| Gradle | 8.10.2 | AGENTS.md, config/ci/github/env.yaml |
| Kotlin | 2.0.21 | config/kotlin/gradle.properties |
| ktlint | 12.1.1 | config/kotlin/gradle.properties |
| Node | 24 | AGENTS.md, config/ci/github/env.yaml |
| Python | 3.10, 3.11 | AGENTS.md |
| JDK | 17 | AGENTS.md |
| Android SDK | 35 | AGENTS.md |
| Android Build Tools | 35.0.0 | AGENTS.md |

## Integration with Existing System

The configuration architecture integrates with:

1. **`scripts/check_harness_drift.py`** - Validates CI commands match documentation
2. **`.github/workflows/ci.yml`** - Uses configured tool versions
3. **`AGENTS.md`** - Config constraints documented
4. **`PLANS.md`** - Config changes tracked as ExecPlan milestones

## Migration from Scattered Configs

If you have existing configuration files scattered throughout the repository:

1. Move them to the appropriate `config/<component>/` directory
2. Update references in build scripts and CI workflows
3. Add symlinks if needed for backward compatibility
4. Update documentation
5. Run `python config/validate.py` to verify

## Configuration Hierarchy

When multiple configurations apply, the priority is:

1. **Component-level** (highest): `crates/*/Cargo.toml`, `kotlin/*/build.gradle.kts`
2. **Workspace-level**: `crates/Cargo.toml`, `kotlin/settings.gradle.kts`
3. **Centralized defaults**: `config/defaults/`
4. **Local overrides**: `config/local/` (not committed)

Component configurations can override centralized defaults, but must maintain consistency with harness enforcement rules.

## Security Considerations

- `config/local/` is gitignored - never commit secrets here
- Use environment variables or secret managers for production secrets
- Security rules in `config/security/` are enforced in CI
- All configurations are validated for syntax and consistency

## Troubleshooting

### "Invalid TOML syntax" error

Ensure your TOML files use valid syntax. Clippy configuration uses a specific format:

```toml
[checks]
unused_variables = "deny"
```

Not:

```toml
clippy::unused_variables = "deny"  # Invalid!
```

### "Missing README.md" warning

These are informational. Add README files to subdirectories for better documentation.

### "Tool version mismatch" error

Update the version in the configuration file to match AGENTS.md specifications.

## References

- [AGENTS.md](AGENTS.md) - Repository harness rules
- [PLANS.md](PLANS.md) - ExecPlan coordination
- [.github/workflows/ci.yml](.github/workflows/ci.yml) - CI workflows
- [docs/architecture/CONFIG_ARCHITECTURE.md](docs/architecture/CONFIG_ARCHITECTURE.md) - Detailed architecture
