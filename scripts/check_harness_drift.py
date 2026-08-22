#!/usr/bin/env python3
"""Detect stale harness docs and unenforced rules.

Checks that are fast, stdlib-only, and CI-suitable:

1. README.md + AGENTS.md code fences mention the canonical verification commands
   that actually appear in .github/workflows/ci.yml (no silent doc drift).
2. Referenced scripts/files mentioned in docs actually exist and are syntactically valid.
3. No forbidden root manifests/lockfiles appear without an ADR.
4. Failure memory docs under docs/failures/*.md have required sections.
5. File-scoped instruction files exist for the polyglot areas.
6. Kotlin commonMain purity is not violated by illegal imports.
7. scripts/autodev-cli.py remains dependency-free / authority-free.
8. PLANS.md preserves the durable ExecPlan coordination contract.

Exit 0 when all checks pass. Exit 1 with a clear message otherwise.
Run: python scripts/check_harness_drift.py
     python scripts/check_harness_drift.py --help
     python scripts/check_harness_drift.py --verbose
"""

from __future__ import annotations

import argparse
import ast
import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CI = ROOT / ".github/workflows/ci.yml"
README = ROOT / "README.md"
AGENTS = ROOT / "AGENTS.md"
PLANS = ROOT / "PLANS.md"

REPRODUCIBLE_SCRIPT = ROOT / "scripts/verify_reproducible.sh"
CANONICAL_CI_FRAGMENTS = [
    "cargo fmt --all -- --check",
    "cargo clippy --workspace --all-targets --all-features -- -D warnings",
    "cargo build --workspace",
    "cargo test --workspace",
    "docker build -f ../Dockerfile -t autodev-server:ci ..",
    "./gradlew clean test",
    ":mpp-core:assemble",
    ":mpp-server:assemble",
    ":mpp-ui:assemble",
    ":mpp-codegraph:assemble",
    ":android-command-center:assembleDebug",
    "ktlintCheck",
    "python -m py_compile",
    "python -m unittest discover -s tests -v",
    "node --check scripts/termux-kanban.mjs",
    "node scripts/termux-kanban.mjs --check",
    'sdkmanager "platforms;android-35" "build-tools;35.0.0"',
]

REQUIRED_PLAN_FRAGMENTS = [
    "## Non-negotiable authority boundary",
    "### Progress",
    "### Surprises & Discoveries",
    "### Decision Log",
    "### Outcomes & Retrospective",
    "An ExecPlan is durable coordination state, not execution authority.",
    "reconciliation",
    "verification",
]

FORBIDDEN_ROOT_FILES = [
    ROOT / "Cargo.toml",
    ROOT / "package.json",
    ROOT / "pyproject.toml",
    ROOT / "requirements.txt",
    ROOT / "kotlin/gradle/libs.versions.toml",
]

INSTRUCTION_FILES = [
    ROOT / ".github/instructions/rust.instructions.md",
    ROOT / ".github/instructions/kotlin.instructions.md",
    ROOT / ".github/instructions/python-node.instructions.md",
]

EXPECTED_FAILURE_SECTIONS = ["## Summary", "## Root Cause", "## Prevention", "## Evidence"]


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def check_ci_exists(errors: list[str], verbose: bool) -> str:
    if not CI.is_file():
        errors.append(f"Missing CI workflow: {CI.relative_to(ROOT)}")
        return ""
    text = _read(CI)
    if verbose:
        print(f"[ok] CI workflow exists: {CI.relative_to(ROOT)}")
    return text


def check_canonical_commands_in_ci(ci_text: str, errors: list[str], verbose: bool) -> None:
    normalized_ci = ci_text.replace(" --locked", "")
    for frag in CANONICAL_CI_FRAGMENTS:
        normalized_frag = frag.replace(" --locked", "")
        if normalized_frag not in normalized_ci and frag not in ci_text:
            errors.append(f"CI drift: expected fragment not found in ci.yml: {frag!r}")


def check_agents_and_readme(ci_text: str, errors: list[str], verbose: bool) -> None:
    for path, label in [(AGENTS, "AGENTS.md"), (README, "README.md")]:
        if not path.is_file():
            errors.append(f"Missing {label} at {path.relative_to(ROOT)}")
            continue
        text = _read(path)
        normalized_text = text.replace(" --locked", "")
        required = (
            CANONICAL_CI_FRAGMENTS
            if path == AGENTS
            else [
                "cargo fmt --all -- --check",
                "cargo build --workspace",
                "cargo test --workspace",
                "cargo clippy --workspace --all-targets -- -D warnings",
            ]
        )
        for frag in required:
            normalized_frag = frag.replace(" --locked", "")
            if normalized_frag not in normalized_text and frag not in text:
                if (
                    path == README
                    and frag == "cargo clippy --workspace --all-targets --all-features -- -D warnings"
                ):
                    if "cargo clippy --workspace --all-targets" not in text:
                        errors.append(f"{label} drift: missing clippy gate fragment {frag!r}")
                else:
                    errors.append(f"{label} drift: missing fragment {frag!r}")
        if path == AGENTS and "kotlin/gradlew" not in text:
            errors.append("AGENTS.md drift: must mention kotlin/gradlew (wrapper-only rule)")
        if verbose and not any(e.startswith(label) for e in errors):
            print(f"[ok] {label} mentions canonical commands")


def check_plans_contract(
    errors: list[str], verbose: bool, plans_path: Path = PLANS
) -> None:
    if not plans_path.is_file():
        errors.append("PLANS.md drift: durable ExecPlan contract is missing")
        return
    text = _read(plans_path)
    missing = [fragment for fragment in REQUIRED_PLAN_FRAGMENTS if fragment not in text]
    for fragment in missing:
        errors.append(f"PLANS.md drift: missing required fragment {fragment!r}")
    if verbose and not missing:
        print("[ok] PLANS.md durable ExecPlan contract")


def check_referenced_files_exist(errors: list[str], verbose: bool) -> None:
    must_exist = [
        ROOT / "crates/Cargo.toml",
        ROOT / "crates/forge-core/Cargo.toml",
        ROOT / "crates/autodev-server/Cargo.toml",
        ROOT / "kotlin/settings.gradle.kts",
        ROOT / "kotlin/build.gradle.kts",
        ROOT / "kotlin/gradle/wrapper/gradle-wrapper.properties",
        ROOT / "kotlin/gradlew",
        ROOT / "scripts/autodev-cli.py",
        ROOT / "scripts/termux-kanban.mjs",
        ROOT / "scripts/build_apk.sh",
        ROOT / "install.py",
        ROOT / "bootstrap_cline_mcp.py",
        ROOT / "Dockerfile",
    ]
    for p in must_exist:
        if not p.exists():
            errors.append(f"Missing referenced file: {p.relative_to(ROOT)}")
        elif verbose:
            print(f"[ok] referenced file exists: {p.relative_to(ROOT)}")


def check_forbidden_files(errors: list[str], verbose: bool) -> None:
    for p in FORBIDDEN_ROOT_FILES:
        if p.exists():
            errors.append(
                f"Forbidden file present: {p.relative_to(ROOT)} — requires ADR before adding; "
                f"see AGENTS.md section 5"
            )
        elif verbose:
            print(f"[ok] forbidden file absent: {p.relative_to(ROOT)}")


def check_instructions(errors: list[str], verbose: bool) -> None:
    for p in INSTRUCTION_FILES:
        if not p.is_file():
            errors.append(f"Missing file-scoped instructions: {p.relative_to(ROOT)}")
            continue
        text = _read(p)
        if "applyTo:" not in text:
            errors.append(f"{p.relative_to(ROOT)} missing applyTo frontmatter")
        if verbose:
            print(f"[ok] instructions present: {p.relative_to(ROOT)}")


def check_failure_memory(errors: list[str], verbose: bool) -> None:
    failures_dir = ROOT / "docs/failures"
    if not failures_dir.is_dir():
        errors.append(f"Missing failure memory directory: {failures_dir.relative_to(ROOT)}")
        return
    mds = sorted(failures_dir.glob("*.md"))
    if not mds:
        errors.append(
            f"No failure docs found in {failures_dir.relative_to(ROOT)}/ — expected at least 1 example"
        )
        return
    for md in mds:
        text = _read(md)
        for section in EXPECTED_FAILURE_SECTIONS:
            if section not in text:
                errors.append(f"{md.relative_to(ROOT)} missing section: {section}")
        if "## Prevention" in text:
            prevention = text.split("## Prevention", 1)[1].split("##", 1)[0]
            if (
                "Detection" not in prevention
                and "detection" not in prevention
                and "check" not in prevention.lower()
            ):
                errors.append(
                    f"{md.relative_to(ROOT)} Prevention must name a Detection/check (or explain why manual)"
                )
        if verbose:
            print(f"[ok] failure doc structure: {md.relative_to(ROOT)}")


def check_kotlin_purity(errors: list[str], verbose: bool) -> None:
    illegal = re.compile(r"^\s*import\s+(java\.|android\.|androidx\.|darwin\.)", re.MULTILINE)
    common_mains = (
        list((ROOT / "kotlin").rglob("src/commonMain/**/*.kt"))
        if (ROOT / "kotlin").exists()
        else []
    )
    if not common_mains:
        common_mains = [
            p for p in (ROOT / "kotlin").rglob("*.kt") if "src/commonMain" in p.as_posix()
        ]
    violations = []
    for kt in common_mains:
        try:
            text = _read(kt)
        except Exception:
            continue
        if illegal.search(text):
            violations.append(kt.relative_to(ROOT).as_posix())
    if violations:
        for violation in violations:
            errors.append(f"Kotlin commonMain purity violation (illegal import) in {violation}")
    elif verbose:
        print(f"[ok] Kotlin commonMain purity: {len(common_mains)} files scanned, no violations")


def check_cli_authority(errors: list[str], verbose: bool) -> None:
    cli = ROOT / "scripts/autodev-cli.py"
    if not cli.is_file():
        errors.append(f"Missing CLI: {cli.relative_to(ROOT)}")
        return
    text = _read(cli)
    forbidden_imports = [
        "import requests",
        "import httpx",
        "import aiohttp",
        "forge_core",
        "AuthorizationGrant",
    ]
    for token in forbidden_imports:
        if token in text:
            errors.append(f"CLI authority drift: forbidden token {token!r} found in {cli.relative_to(ROOT)}")
    if "urllib" not in text:
        errors.append(f"CLI drift: {cli.relative_to(ROOT)} should use urllib (stdlib-only)")
    try:
        ast.parse(text)
    except SyntaxError as exc:
        errors.append(f"CLI syntax error: {exc}")
    if verbose and not any("CLI" in error for error in errors):
        print(f"[ok] CLI authority boundary: {cli.relative_to(ROOT)} stdlib-only")


def check_reproducible_script(errors: list[str], verbose: bool) -> None:
    script = REPRODUCIBLE_SCRIPT
    if not script.is_file():
        errors.append(
            f"Missing reproducible verification script: {script.relative_to(ROOT)} — see docs/failures/002"
        )
        return
    if not os.access(script, os.X_OK):
        errors.append(
            f"Reproducible script not executable: {script.relative_to(ROOT)} — run chmod +x {script.relative_to(ROOT)}"
        )
    try:
        text = _read(script)
    except Exception as exc:
        errors.append(f"Cannot read {script.relative_to(ROOT)}: {exc}")
        return
    if "cargo fmt --all -- --check" not in text:
        errors.append(f"{script.relative_to(ROOT)} must contain cargo fmt gate")
    if "check_harness_drift" not in text:
        errors.append(f"{script.relative_to(ROOT)} must invoke check_harness_drift")
    try:
        agents_text = _read(AGENTS)
    except Exception:
        agents_text = ""
    if "verify_reproducible.sh" not in agents_text:
        errors.append(
            "AGENTS.md drift: must mention scripts/verify_reproducible.sh (Slice A offline-capable gate)"
        )
    if verbose and not any("reproducible" in error.lower() for error in errors):
        print(f"[ok] reproducible script: {script.relative_to(ROOT)} executable and referenced")


def check_scripts_syntax(errors: list[str], verbose: bool) -> None:
    for py in [
        ROOT / "scripts/check_harness_drift.py",
        ROOT / "install.py",
        ROOT / "bootstrap_cline_mcp.py",
    ]:
        if not py.is_file():
            continue
        try:
            ast.parse(_read(py))
        except SyntaxError as exc:
            errors.append(f"Syntax error in {py.relative_to(ROOT)}: {exc}")
        else:
            if verbose:
                print(f"[ok] Python syntax: {py.relative_to(ROOT)}")
    termux = ROOT / "scripts/termux-kanban.mjs"
    if termux.is_file():
        import shutil
        import subprocess

        node = shutil.which("node")
        if node:
            result = subprocess.run([node, "--check", str(termux)], capture_output=True, text=True)
            if result.returncode != 0:
                errors.append(f"Node syntax error in {termux.relative_to(ROOT)}: {result.stderr.strip()}")
            elif verbose:
                print(f"[ok] Node syntax: {termux.relative_to(ROOT)}")


def check_config_parity(errors: list[str], verbose: bool) -> None:
    """Copies of centralized config files must stay identical to their source.

    Gradle reads kotlin/gradle.properties from the project root; the
    centralized file in config/kotlin/ is the single source of truth (see
    docs/architecture/KOTLIN_CONFIG_INTEGRATION.md). Silent divergence between
    the two is exactly how the $JDK_17_HOME / enableBuildCache defects
    propagated in cycle 2026-08-21-kotlin-mpp - fail closed instead.
    """
    parity_pairs = [
        (ROOT / "config/kotlin/gradle.properties", ROOT / "kotlin/gradle.properties"),
    ]
    for source, consumer in parity_pairs:
        if not source.is_file() or not consumer.is_file():
            continue  # optional pair; missing files are covered elsewhere
        if _read(source) != _read(consumer):
            errors.append(
                "Config parity drift: "
                f"{consumer.relative_to(ROOT)} differs from "
                f"{source.relative_to(ROOT)} - regenerate the copy from the "
                "centralized source (see docs/architecture/KOTLIN_CONFIG_INTEGRATION.md)"
            )
        elif verbose:
            print(f"[ok] Config parity: {consumer.relative_to(ROOT)} matches source")


def check_amcx1(errors: list[str], verbose: bool) -> None:
    """AMCX-1 v1.1 state validation (opt-in via --amcx1).

    Structural, stdlib-only validation of .vibe/ state files against the
    tracked draft-07 schemas in schemas/ (required fields, digest format,
    duplicate logical_identity). Skips silently when no AMCX-1 state exists,
    so worktrees without agent-memory state are unaffected.
    """
    digest_re = re.compile(r"^[0-9a-f]{64}$")
    amx_state = ROOT / ".vibe/amcx-memory/state.json"
    ecm_state = ROOT / ".vibe/ecm-state.json"
    if not amx_state.is_file() and not ecm_state.is_file():
        if verbose:
            print("[ok] AMCX-1: no .vibe state present (nothing to validate)")
        return

    required_amx = [
        "schema_version", "origin", "logical_identity", "repository_scope",
        "provenance", "causal_ancestry", "trust_validity_state", "visibility",
        "purpose", "retraction_deletion_barriers", "canonical_semantic_digest",
    ]

    if amx_state.is_file():
        try:
            data = json.loads(_read(amx_state))
        except json.JSONDecodeError as exc:
            errors.append(f"AMCX-1: {amx_state.relative_to(ROOT)} is not valid JSON: {exc}")
            return
        memories = data.get("memories", [])
        seen: dict[str, int] = {}
        for index, memory in enumerate(memories):
            where = f"{amx_state.relative_to(ROOT)}#memories[{index}]"
            # Presence check: empty objects (e.g. repository_scope={}) are
            # schema-valid; the Engram server auto-populates them as {}.
            missing = [f for f in required_amx if f not in memory]
            if missing:
                errors.append(f"AMCX-1: {where} missing required fields: {missing}")
            digest = memory.get("canonical_semantic_digest", "")
            if digest and not digest_re.match(str(digest)):
                errors.append(
                    f"AMCX-1: {where} canonical_semantic_digest is not 64-char lowercase hex"
                )
            logical_id = memory.get("logical_identity")
            if logical_id:
                if logical_id in seen:
                    errors.append(
                        f"AMCX-1: duplicate logical_identity {logical_id!r} at "
                        f"memories[{index}] (first seen at memories[{seen[logical_id]}])"
                    )
                else:
                    seen[logical_id] = index
        if verbose:
            print(f"[ok] AMCX-1: {amx_state.relative_to(ROOT)} - {len(memories)} memories, {len(seen)} unique identities")

    if ecm_state.is_file():
        try:
            data = json.loads(_read(ecm_state))
        except json.JSONDecodeError as exc:
            errors.append(f"AMCX-1: {ecm_state.relative_to(ROOT)} is not valid JSON: {exc}")
            return
        entries = data.get("entries", data if isinstance(data, list) else [])
        combos: set[tuple] = set()
        for index, entry in enumerate(entries):
            if not isinstance(entry, dict):
                continue
            combo = (entry.get("task_id"), entry.get("attempt_id"))
            if combo in combos:
                errors.append(
                    f"AMCX-1: duplicate ECM pattern (task_id, attempt_id)={combo} "
                    f"at {ecm_state.relative_to(ROOT)} entries[{index}]"
                )
            elif all(combo):
                combos.add(combo)
        if verbose:
            print(f"[ok] AMCX-1: {ecm_state.relative_to(ROOT)} - {len(entries)} entries")


def check_workspace_members(errors: list[str], verbose: bool) -> None:
    """AGENTS.md must document every Rust workspace crate member.

    Fails closed when crates/Cargo.toml is unreadable or has no parseable
    members array, so a restructured workspace cannot silently escape docs.
    """
    cargo = ROOT / "crates/Cargo.toml"
    if not cargo.is_file():
        errors.append("Harness drift: crates/Cargo.toml is missing")
        return
    text = _read(cargo)
    match = re.search(r"^members\s*=\s*\[(.*?)\]", text, re.S | re.M)
    if not match:
        errors.append("Harness drift: crates/Cargo.toml has no parseable workspace.members array")
        return
    members = re.findall(r'"([^"]+)"', match.group(1))
    agents_text = _read(AGENTS)
    for member in members:
        name = member.split("/")[0].strip()
        if not name:
            continue
        if f"`{name}`" not in agents_text:
            errors.append(
                f"Harness drift: Rust workspace member '{member}' is not documented "
                "in AGENTS.md (section 4, Rust bullet)"
            )
        elif verbose:
            print(f"[ok] workspace member documented in AGENTS.md: {name}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Check harness drift for AutoDev")
    parser.add_argument("--verbose", action="store_true", help="print passing checks")
    parser.add_argument(
        "--amcx1", action="store_true",
        help="also validate AMCX-1 .vibe state files (skipped when absent)",
    )
    args = parser.parse_args()

    errors: list[str] = []

    ci_text = check_ci_exists(errors, args.verbose)
    if ci_text:
        check_canonical_commands_in_ci(ci_text, errors, args.verbose)
        check_agents_and_readme(ci_text, errors, args.verbose)

    check_plans_contract(errors, args.verbose)
    check_referenced_files_exist(errors, args.verbose)
    check_forbidden_files(errors, args.verbose)
    check_workspace_members(errors, args.verbose)
    check_instructions(errors, args.verbose)
    check_failure_memory(errors, args.verbose)
    check_kotlin_purity(errors, args.verbose)
    check_config_parity(errors, args.verbose)
    check_cli_authority(errors, args.verbose)
    check_reproducible_script(errors, args.verbose)
    check_scripts_syntax(errors, args.verbose)
    if args.amcx1:
        check_amcx1(errors, args.verbose)

    if errors:
        print("Harness drift detected:", file=sys.stderr)
        for index, error in enumerate(errors, 1):
            print(f"  {index}. {error}", file=sys.stderr)
        print(
            "\nFix guidance: update AGENTS.md / README.md / .github/workflows/ci.yml so all three agree, "
            "or remove the forbidden file. See AGENTS.md section 9.",
            file=sys.stderr,
        )
        return 1

    print("Harness drift check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
