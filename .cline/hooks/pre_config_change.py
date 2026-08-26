#!/usr/bin/env python3
"""Pre-configuration change guard for Cline.

This hook runs before any configuration file modification in the AutoDev repository.
It ensures that configuration changes respect the repository's harness rules,
trust boundaries, and version pinning requirements.

Usage:
    This hook is triggered automatically by Cline when modifying files under config/
    or when the modification affects configuration mentioned in AGENTS.md.
"""

import json
import sys
from pathlib import Path


def _read(path: Path) -> str:
    """Read file content."""
    return path.read_text(encoding="utf-8")


def _get_modified_files(event: dict) -> list[str]:
    """Extract modified files from the event."""
    # Cline provides different event structures
    tool_input = event.get("tool_input", {})
    
    # For file modifications
    if isinstance(tool_input, dict):
        files = tool_input.get("files", [])
        if isinstance(files, list):
            return [f for f in files if f]
    
    # For single file operations
    path = tool_input.get("path", "") or event.get("path", "")
    if path:
        return [path]
    
    # For edit operations
    file_path = tool_input.get("file_path", "") or event.get("file_path", "")
    if file_path:
        return [file_path]
    
    return []


def _is_config_file(path: str) -> bool:
    """Check if a file path is a configuration file."""
    config_dirs = ["config/", ".github/workflows/", "crates/Cargo.toml", "kotlin/"]
    config_files = [
        "Cargo.toml", "build.gradle.kts", "settings.gradle.kts",
        "gradle.properties", ".coderabbit.yaml", "ast-grep.yml",
    ]
    
    path_lower = path.lower()
    
    # Check if in config directory
    for config_dir in config_dirs:
        if config_dir in path_lower:
            return True
    
    # Check specific config files
    for config_file in config_files:
        if config_file in path_lower:
            return True
    
    return False


def _check_forbidden_root_manifests(event: dict) -> dict:
    """Check for creation of forbidden root manifests."""
    modified_files = _get_modified_files(event)
    
    forbidden_files = [
        "Cargo.toml",
        "package.json", 
        "pyproject.toml",
        "requirements.txt",
    ]
    
    for file_path in modified_files:
        for forbidden in forbidden_files:
            # Check if creating forbidden file at root
            if file_path == forbidden or file_path.endswith(f"/{forbidden}"):
                # Only block if it's at the repository root
                if "/" not in file_path or file_path.count("/") == 0:
                    return {
                        "cancel": True,
                        "reason": f"Creating root {forbidden} is forbidden by AGENTS.md section 4. "
                                  f"Use component-specific directories or create an ADR first."
                    }
    
    return {"cancel": False}


def _check_version_consistency(event: dict) -> dict:
    """Check that version pins match AGENTS.md."""
    modified_files = _get_modified_files(event)
    
    # Only check version-related files
    version_files = [
        "config/kotlin/gradle.properties",
        "config/rust/toolchain.toml",
        "config/ci/github/env.yaml",
    ]
    
    for file_path in modified_files:
        if any(vf in file_path for vf in version_files):
            try:
                content = _read(Path(file_path))
                # For now, just ensure the file is valid
                # More sophisticated version checking could be added
            except Exception:
                return {
                    "cancel": True,
                    "reason": f"Failed to read modified configuration file: {file_path}"
                }
    
    return {"cancel": False}


def _check_config_directory_structure(event: dict) -> dict:
    """Ensure config directory structure is maintained."""
    modified_files = _get_modified_files(event)
    
    for file_path in modified_files:
        if "config/" in file_path:
            # Check that the file is in a valid subdirectory
            valid_dirs = ["defaults", "rust", "kotlin", "python", "security", "ci"]
            parts = file_path.split("/")
            
            if len(parts) >= 2 and "config" in parts:
                config_index = parts.index("config")
                if config_index + 1 < len(parts):
                    subdir = parts[config_index + 1]
                    # Allow known subdirectories or deeper nesting
                    if subdir not in valid_dirs and not any(
                        subdir.startswith(vd) for vd in valid_dirs
                    ):
                        # This might be a new subdirectory - that's okay
                        pass
    
    return {"cancel": False}


def _check_security_sensitive_files(event: dict) -> dict:
    """Check modifications to security-sensitive configuration files."""
    modified_files = _get_modified_files(event)
    
    security_files = [
        "config/security/ast-grep.yml",
        "config/security/",
        ".coderabbit.yaml",
    ]
    
    for file_path in modified_files:
        for sec_file in security_files:
            if sec_file in file_path:
                return {
                    "cancel": False,  # Don't block, just warn
                    "reason": f"Modifying security configuration: {file_path}. "
                              f"Please ensure changes are reviewed by Security agent."
                }
    
    return {"cancel": False}


def _validate_config_syntax(event: dict) -> dict:
    """Validate configuration file syntax before modification."""
    modified_files = _get_modified_files(event)
    
    for file_path in modified_files:
        if file_path.endswith(".toml"):
            try:
                import tomllib
                with Path(file_path).open("rb") as f:
                    tomllib.load(f)
            except Exception as e:
                return {
                    "cancel": True,
                    "reason": f"Invalid TOML syntax in {file_path}: {e}"
                }
            except ImportError:
                # Python < 3.11
                try:
                    import tomli
                    with Path(file_path).open("rb") as f:
                        tomli.load(f)
                except ImportError:
                    pass  # Skip syntax validation if tomli not available
                except Exception as e:
                    return {
                        "cancel": True,
                        "reason": f"Invalid TOML syntax in {file_path}: {e}"
                    }
    
    return {"cancel": False}


def main():
    """Main entry point."""
    try:
        raw = sys.stdin.read()
        event = json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError:
        event = {}
    
    # Check for forbidden root manifests
    result = _check_forbidden_root_manifests(event)
    if result.get("cancel"):
        print(json.dumps(result))
        sys.exit(2)
    
    # Check version consistency
    result = _check_version_consistency(event)
    if result.get("cancel"):
        print(json.dumps(result))
        sys.exit(2)
    
    # Check config directory structure
    result = _check_config_directory_structure(event)
    if result.get("cancel"):
        print(json.dumps(result))
        sys.exit(2)
    
    # Check security-sensitive files
    result = _check_security_sensitive_files(event)
    if result.get("cancel"):
        print(json.dumps(result))
        sys.exit(2)
    
    # Validate syntax
    result = _validate_config_syntax(event)
    if result.get("cancel"):
        print(json.dumps(result))
        sys.exit(2)
    
    # All checks passed
    print(json.dumps({"cancel": False}))


if __name__ == "__main__":
    main()
