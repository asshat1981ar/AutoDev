# AutoDev Configuration Directory

This directory contains centralized configuration for the AutoDev multi-agent runtime.

## Structure

```
config/
├── README.md                    # This file
├── defaults/                    # Shared default configurations
│   ├── clippy.toml              # Rust clippy lint defaults
│   ├── rustfmt.toml             # Rust formatting defaults
│   └── common/                  # Cross-language configs
│       └── .editorconfig        # Editor configuration
├── rust/                       # Rust workspace configuration
│   ├── toolchain.toml           # Rust toolchain specification
│   └── features/                # Feature flag configurations
│       └── forge-core.toml      # ForgeCore feature flags
├── kotlin/                     # Kotlin workspace configuration
│   ├── gradle.properties        # Shared Gradle properties
│   └── ktlint/                  # Ktlint configuration
│       └── .ktlint.yaml         # Ktlint rules
├── python/                     # Python configuration
│   ├── mypy.ini                 # Static type checking
│   ├── pytest.ini               # Test framework
│   └── unittest.ini              # Unittest defaults
├── security/                   # Security scanning configurations
│   ├── ast-grep.yml             # Ast-grep rules
│   ├── ast-grep/                # Additional ast-grep configs
│   │   └── rules/               # Custom rules directory
│   └── coderabbit.yaml          # CodeRabbit review instructions
└── ci/                         # CI-specific configurations
    ├── github/                  # GitHub Actions
    │   └── env.yaml             # Environment variables
    └── local/                   # Local development (gitignored)
        └── .env.example          # Example local overrides
```

## Usage

### Rust Development

From `crates/` directory:

```bash
# Use centralized clippy config
cargo clippy --workspace --all-targets --all-features -- -D warnings -c ../config/defaults/clippy.toml

# Use centralized rustfmt config  
cargo fmt --all -- --config-path ../config/defaults/rustfmt.toml
```

### Kotlin Development

From `kotlin/` directory:

```bash
# Gradle uses gradle.properties from config/kotlin/
./gradlew build --no-daemon
```

### Python Development

From repo root:

```bash
# Type checking with centralized config
mypy --config-file config/python/mypy.ini scripts/ install.py

# Testing with centralized config
python -m pytest --pytest.ini config/python/pytest.ini
```

## Configuration Hierarchy

1. **Component-level** (highest priority): `crates/*/Cargo.toml`, `kotlin/*/build.gradle.kts`
2. **Workspace-level**: `crates/Cargo.toml`, `kotlin/settings.gradle.kts`
3. **Centralized defaults** (lowest priority): `config/defaults/`

Component configurations can override centralized defaults, but must maintain consistency with harness enforcement rules.

## Harness Integration

All configurations are validated by:
- `scripts/check_harness_drift.py` - Ensures CI commands match documented commands
- `.github/workflows/ci.yml` - Uses configured tool versions
- Component build systems - Validate their own configurations

## Adding New Configuration

1. Create the configuration file in the appropriate subdirectory
2. Document it in this README
3. Update `scripts/check_harness_drift.py` to validate it
4. Update `.github/workflows/ci.yml` to use it
5. Ensure it respects the rules in `AGENTS.md` section 4

## Local Overrides

Create a `config/local/` directory (add to `.gitignore`):

```bash
mkdir -p config/local
echo "config/local/" >> .gitignore
```

Place local-specific configurations in this directory. They take precedence over committed configs but are never committed to the repository.

## Version Pinning

All tool versions must be explicitly pinned:
- Rust toolchain: `config/rust/toolchain.toml`
- Kotlin: `config/kotlin/gradle.properties`
- Python: Not applicable (stdlib only per AGENTS.md)
- Node: Managed by system, pinned in CI

## Security

Security scanning configurations live in `config/security/`. These define:
- Code patterns that violate trust boundaries
- Required approval gates
- Secret detection rules

See `config/security/README.md` for details.
