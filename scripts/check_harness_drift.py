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


def main() -> int:
    parser = argparse.ArgumentParser(description="Check harness drift for AutoDev")
    parser.add_argument("--verbose", action="store_true", help="print passing checks")
    args = parser.parse_args()

    errors: list[str] = []

    ci_text = check_ci_exists(errors, args.verbose)
    if ci_text:
        check_canonical_commands_in_ci(ci_text, errors, args.verbose)
        check_agents_and_readme(ci_text, errors, args.verbose)

    check_plans_contract(errors, args.verbose)
    check_referenced_files_exist(errors, args.verbose)
    check_forbidden_files(errors, args.verbose)
    check_instructions(errors, args.verbose)
    check_failure_memory(errors, args.verbose)
    check_kotlin_purity(errors, args.verbose)
    check_cli_authority(errors, args.verbose)
    check_reproducible_script(errors, args.verbose)
    check_scripts_syntax(errors, args.verbose)

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
