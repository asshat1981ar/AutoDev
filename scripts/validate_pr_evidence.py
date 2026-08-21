#!/usr/bin/env python3
"""Validate submitted pull-request verification evidence from a GitHub event."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

HTML_COMMENT_RE = re.compile(r"<!--.*?-->", re.DOTALL)
TOP_LEVEL_NO_IMPACT_RE = re.compile(
    r"^- \[([ xX])\] No trusted execution or authorization changes\s*$",
    re.MULTILINE,
)
TOP_LEVEL_IMPACT_RE = re.compile(
    r"^- \[([ xX])\] Trusted-boundary impact present(?:\s*\(.*\))?\s*$",
    re.MULTILINE,
)
NESTED_BOUNDARY_RE = re.compile(r"^\s{2,}- \[([xX])\] .+changed\s*$", re.MULTILINE)


def _strip_comments(text: str) -> str:
    """Remove HTML comments before evaluating user-visible PR evidence."""
    return HTML_COMMENT_RE.sub("", text)


def _section(body: str, heading: str) -> str | None:
    """Return the visible Markdown level-three section body for a heading."""
    pattern = re.compile(
        rf"^### {re.escape(heading)}\s*$\n(?P<content>.*?)(?=^#{{1,3}} \S|\Z)",
        re.MULTILINE | re.DOTALL,
    )
    match = pattern.search(body)
    if not match:
        return None
    return match.group("content").strip()


def _checked(match: re.Match[str] | None) -> bool:
    """Return whether a matched Markdown checkbox is checked."""
    return bool(match and match.group(1).lower() == "x")


def validate_body(body: str | None) -> list[str]:
    """Return validation errors for a submitted pull-request body."""
    if not isinstance(body, str) or not body.strip():
        return ["Pull-request body is required for verification evidence."]

    visible_body = _strip_comments(body)
    errors: list[str] = []
    no_impact = TOP_LEVEL_NO_IMPACT_RE.search(visible_body)
    impact = TOP_LEVEL_IMPACT_RE.search(visible_body)
    checked_states = int(_checked(no_impact)) + int(_checked(impact))
    if checked_states != 1:
        errors.append("Trust-boundary impact must select exactly one top-level declaration.")
    if _checked(no_impact) and NESTED_BOUNDARY_RE.search(visible_body):
        errors.append("No-impact declaration cannot be combined with checked boundary categories.")

    commands = _section(visible_body, "Commands run")
    if not commands:
        errors.append("Commands run section must contain the exact verification commands executed.")

    evidence = _section(visible_body, "Evidence")
    if not evidence:
        errors.append("Evidence section must contain CI, artifact, or command-output evidence.")
    else:
        label_match = re.search(
            r"CI run / artifact / output:\s*(?P<value>.*)", evidence, re.DOTALL
        )
        if not label_match or not label_match.group("value").strip():
            errors.append("Evidence must provide a value after 'CI run / artifact / output:'.")

    return errors


def validate_event(event_path: Path) -> list[str]:
    """Load a GitHub pull_request event and validate its submitted body."""
    try:
        event = json.loads(event_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return [f"Unable to read GitHub event JSON: {exc}"]

    pull_request = event.get("pull_request")
    if not isinstance(pull_request, dict):
        return ["GitHub event does not contain a pull_request object."]
    return validate_body(pull_request.get("body"))


def main(argv: list[str]) -> int:
    """CLI entry point returning non-zero when submitted evidence is invalid."""
    if len(argv) != 2:
        print("usage: validate_pr_evidence.py <github-event-json>", file=sys.stderr)
        return 2

    errors = validate_event(Path(argv[1]))
    if errors:
        for error in errors:
            print(f"PR evidence error: {error}", file=sys.stderr)
        return 1

    print("pull-request evidence: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
