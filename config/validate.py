#!/usr/bin/env python3
"""
AutoDev Configuration Validator

This script validates that all configurations in the config/ directory
are consistent with the repository's harness rules defined in AGENTS.md.

Usage:
    python config/validate.py [--verbose] [--fix]

Exit codes:
    0 - All configurations are valid
    1 - Validation failures found
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple


# AutoDev repository root
REPO_ROOT = Path(__file__).parent.parent.resolve()
CONFIG_DIR = REPO_ROOT / "config"
AGENTS_MD = REPO_ROOT / "AGENTS.md"
CI_YML = REPO_ROOT / ".github" / "workflows" / "ci.yml"


class ValidationError:
    """Represents a configuration validation error."""

    def __init__(self, severity: str, file_path: str, line: int, message: str):
        self.severity = severity
        self.file_path = file_path
        self.line = line
        self.message = message

    def __str__(self) -> str:
        return f"[{self.severity.upper()}] {self.file_path}:{self.line}: {self.message}"


class ConfigValidator:
    """Validates AutoDev configuration files."""

    def __init__(self, verbose: bool = False, fix: bool = False):
        self.verbose = verbose
        self.fix = fix
        self.errors: List[ValidationError] = []
        self.warnings: List[ValidationError] = []

    def log(self, message: str) -> None:
        """Log a message if verbose mode is enabled."""
        if self.verbose:
            print(f"[INFO] {message}")

    def add_error(self, file_path: str, line: int, message: str) -> None:
        """Add a validation error."""
        self.errors.append(ValidationError("error", file_path, line, message))
        if self.verbose:
            print(f"[ERROR] {file_path}:{line}: {message}")

    def add_warning(self, file_path: str, line: int, message: str) -> None:
        """Add a validation warning."""
        self.warnings.append(ValidationError("warning", file_path, line, message))
        if self.verbose:
            print(f"[WARNING] {file_path}:{line}: {message}")

    def validate_structure(self) -> None:
        """Validate the configuration directory structure."""
        self.log(f"Validating configuration directory structure...")

        required_dirs = [
            "defaults",
            "rust",
            "kotlin",
            "python",
            "security",
            "ci",
        ]

        for req_dir in required_dirs:
            path = CONFIG_DIR / req_dir
            if not path.exists():
                self.add_error(
                    str(CONFIG_DIR),
                    0,
                    f"Required directory '{req_dir}/' is missing",
                )

        # Check for README in each directory
        for req_dir in required_dirs:
            readme = CONFIG_DIR / req_dir / "README.md"
            if not readme.exists():
                self.add_warning(
                    str(CONFIG_DIR / req_dir),
                    0,
                    f"Missing README.md in {req_dir}/",
                )

    def validate_no_root_manifests(self) -> None:
        """Validate that no root manifests exist (AGENTS.md rule)."""
        self.log("Checking for forbidden root manifests...")

        forbidden_files = ["Cargo.toml", "package.json", "pyproject.toml"]

        for forbidden in forbidden_files:
            path = REPO_ROOT / forbidden
            if path.exists():
                self.add_error(
                    str(path),
                    0,
                    f"Root {forbidden} is forbidden by AGENTS.md section 4",
                )

    def validate_tool_versions(self) -> None:
        """Validate that tool versions match AGENTS.md specifications."""
        self.log("Validating tool versions...")

        # Read AGENTS.md for version specifications
        agents_content = AGENTS_MD.read_text()

        # Expected versions from AGENTS.md
        expected_versions = {
            "rustfmt": "rustfmt",
            "clippy": "clippy",
            "cargo": "cargo",
            "gradle": "8.10.2",
            "kotlin": "2.0.21",
            "ktlint": "12.1.1",
            "node": "24",
            "python": "3.10 or 3.11",
            "jdk": "17",
            "android_sdk": "35",
            "android_build_tools": "35.0.0",
        }

        # Check Rust toolchain config
        toolchain_path = CONFIG_DIR / "rust" / "toolchain.toml"
        if toolchain_path.exists():
            content = toolchain_path.read_text()
            if "channel = \"stable\"" not in content:
                self.add_error(
                    str(toolchain_path),
                    0,
                    "Rust toolchain should use stable channel",
                )

        # Check Kotlin gradle.properties
        gradle_props = CONFIG_DIR / "kotlin" / "gradle.properties"
        if gradle_props.exists():
            content = gradle_props.read_text()
            if "kotlin.version=2.0.21" not in content:
                self.add_error(
                    str(gradle_props),
                    0,
                    "Kotlin version should be 2.0.21",
                )
            if "ktlint.version=12.1.1" not in content:
                self.add_error(
                    str(gradle_props),
                    0,
                    "ktlint version should be 12.1.1",
                )
            if "jvm.target=17" not in content:
                self.add_error(
                    str(gradle_props),
                    0,
                    "JVM target should be 17",
                )

        # Check CI env config
        ci_env = CONFIG_DIR / "ci" / "github" / "env.yaml"
        if ci_env.exists():
            content = ci_env.read_text()
            if "GRADLE_VERSION: 8.10.2" not in content:
                self.add_error(
                    str(ci_env),
                    0,
                    "Gradle version should be 8.10.2",
                )
            if "NODE_VERSION: 24" not in content:
                self.add_error(
                    str(ci_env),
                    0,
                    "Node version should be 24",
                )

    def validate_line_length(self) -> None:
        """Validate that line length limits are consistent across configs."""
        self.log("Validating line length consistency...")

        expected_max = 100

        configs_to_check = [
            (CONFIG_DIR / "defaults" / "rustfmt.toml", "max_width"),
            (CONFIG_DIR / "defaults" / "common" / ".editorconfig", "max_line_length"),
            (CONFIG_DIR / "kotlin" / "ktlint" / ".ktlint.yaml", "max_line_length"),
        ]

        for config_path, setting in configs_to_check:
            if config_path.exists():
                content = config_path.read_text()
                if f"{setting} = {expected_max}" not in content and \
                   f"{setting}: {expected_max}" not in content:
                    self.add_warning(
                        str(config_path),
                        0,
                        f"Expected {setting} = {expected_max} for consistency",
                    )

    def validate_symlinks(self) -> None:
        """Validate that symlinks are working correctly."""
        self.log("Checking symlinks...")

        # Check if .editorconfig should be symlinked
        editorconfig_source = CONFIG_DIR / "defaults" / "common" / ".editorconfig"
        editorconfig_root = REPO_ROOT / ".editorconfig"

        if editorconfig_root.exists():
            if not editorconfig_root.is_symlink():
                self.add_warning(
                    str(editorconfig_root),
                    0,
                    "Consider symlinking .editorconfig to config/defaults/common/.editorconfig",
                )

    def validate_gitignore(self) -> None:
        """Validate that config/local/ is properly gitignored."""
        self.log("Checking .gitignore...")

        gitignore = REPO_ROOT / ".gitignore"
        if gitignore.exists():
            content = gitignore.read_text()
            if "config/local/" not in content:
                self.add_error(
                    str(gitignore),
                    0,
                    "config/local/ should be in .gitignore",
                )

    def validate_harness_consistency(self) -> None:
        """Validate that configs match harness rules from AGENTS.md."""
        self.log("Checking harness consistency...")

        # Read AGENTS.md
        if not AGENTS_MD.exists():
            self.add_error(str(AGENTS_MD), 0, "AGENTS.md not found")
            return

        agents_content = AGENTS_MD.read_text()

        # Check for mentioned tools
        tools_in_agents = [
            "rustfmt",
            "clippy",
            "ktlint",
            "cargo",
            "gradle",
            "python",
            "node",
        ]

        for tool in tools_in_agents:
            if tool not in agents_content:
                self.add_warning(
                    str(AGENTS_MD),
                    0,
                    f"Tool '{tool}' mentioned in config but not in AGENTS.md",
                )

    def validate_file_syntax(self) -> None:
        """Validate syntax of configuration files."""
        self.log("Validating file syntax...")

        # Check TOML files
        for toml_file in CONFIG_DIR.rglob("*.toml"):
            self._validate_toml(toml_file)

        # Check YAML files
        for yaml_file in CONFIG_DIR.rglob("*.yaml"):
            self._validate_yaml(yaml_file)

        # Check YML files
        for yml_file in CONFIG_DIR.rglob("*.yml"):
            self._validate_yaml(yml_file)

    def _validate_toml(self, path: Path) -> None:
        """Validate a TOML file."""
        try:
            import tomllib

            with path.open("rb") as f:
                tomllib.load(f)
        except ImportError:
            # Python < 3.11
            try:
                import tomli

                with path.open("rb") as f:
                    tomli.load(f)
            except (ImportError, Exception) as e:
                self.add_warning(str(path), 0, f"Cannot validate TOML: {e}")
        except Exception as e:
            self.add_error(str(path), 0, f"Invalid TOML syntax: {e}")

    def _validate_yaml(self, path: Path) -> None:
        """Validate a YAML file."""
        try:
            import yaml

            with path.open() as f:
                yaml.safe_load(f)
        except ImportError:
            # Skip if PyYAML not installed
            pass
        except Exception as e:
            self.add_error(str(path), 0, f"Invalid YAML syntax: {e}")

    def run_all_checks(self) -> Tuple[int, int]:
        """Run all validation checks."""
        self.log("Starting configuration validation...")
        self.log(f"Repository root: {REPO_ROOT}")
        self.log(f"Config directory: {CONFIG_DIR}")

        # Run all validation methods
        self.validate_structure()
        self.validate_no_root_manifests()
        self.validate_tool_versions()
        self.validate_line_length()
        self.validate_symlinks()
        self.validate_gitignore()
        self.validate_harness_consistency()
        self.validate_file_syntax()

        return len(self.errors), len(self.warnings)

    def print_results(self) -> None:
        """Print validation results."""
        total_errors = len(self.errors)
        total_warnings = len(self.warnings)

        print("\n" + "=" * 60)
        print("VALIDATION RESULTS")
        print("=" * 60)

        if self.errors:
            print(f"\n[ERROR] {total_errors} error(s) found:\n")
            for error in self.errors:
                print(f"  {error}")

        if self.warnings:
            print(f"\n[WARNING] {total_warnings} warning(s) found:\n")
            for warning in self.warnings:
                print(f"  {warning}")

        if total_errors == 0 and total_warnings == 0:
            print("\n[SUCCESS] All configurations are valid!")
        else:
            print(f"\n[SUMMARY] {total_errors} error(s), {total_warnings} warning(s)")

        print("=" * 60)


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Validate AutoDev configuration files"
    )
    parser.add_argument(
        "--verbose", "-v", action="store_true", help="Enable verbose output"
    )
    parser.add_argument(
        "--fix", "-f", action="store_true", help="Attempt to fix issues"
    )
    parser.add_argument(
        "--strict", "-s", action="store_true", help="Treat warnings as errors"
    )

    args = parser.parse_args()

    validator = ConfigValidator(verbose=args.verbose, fix=args.fix)
    error_count, warning_count = validator.run_all_checks()
    validator.print_results()

    # Determine exit code
    if args.strict:
        exit_code = 1 if (error_count + warning_count) > 0 else 0
    else:
        exit_code = 1 if error_count > 0 else 0

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
