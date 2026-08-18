#!/usr/bin/env python3
"""Deterministic changed-path overlap evidence for concurrent pull requests.

This module is intentionally pure and unprivileged. It accepts already-observed
PR changed-path sets and returns deterministic collision evidence; it does not
fetch GitHub data, mutate branches, block merges, or grant any authority.
"""

from __future__ import annotations

import argparse
import json
from collections import defaultdict
from pathlib import Path
from typing import Iterable, Mapping


def _normalize_path(path: str) -> str:
    normalized = path.replace("\\", "/")
    while normalized.startswith("./"):
        normalized = normalized[2:]
    return normalized


def classify_path(path: str) -> str:
    normalized = _normalize_path(path)
    if normalized.startswith("crates/forge-core/"):
        return "authority-core"
    if normalized.startswith(".github/workflows/"):
        return "ci-governance"
    if normalized.startswith("docs/") or normalized.lower().endswith(".md"):
        return "documentation"
    return "general"


def _severity(classification: str, pr_count: int) -> str:
    if classification in {"authority-core", "ci-governance"}:
        return "high"
    if pr_count >= 3:
        return "medium"
    return "low"


def analyze_pr_overlaps(changed_paths: Mapping[int, Iterable[str]]) -> dict:
    """Return stable evidence for paths changed by two or more pull requests."""
    owners: dict[str, set[int]] = defaultdict(set)
    pull_requests = sorted(int(number) for number in changed_paths)

    for number, paths in changed_paths.items():
        pr_number = int(number)
        for path in set(paths):
            normalized = _normalize_path(path)
            if normalized:
                owners[normalized].add(pr_number)

    collisions = []
    for path in sorted(owners):
        prs = sorted(owners[path])
        if len(prs) < 2:
            continue
        classification = classify_path(path)
        collisions.append(
            {
                "path": path,
                "classification": classification,
                "pull_requests": prs,
                "severity": _severity(classification, len(prs)),
            }
        )

    return {
        "schema_version": 1,
        "pull_requests": pull_requests,
        "collision_count": len(collisions),
        "collisions": collisions,
    }


def _load(path: Path) -> dict[int, list[str]]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise ValueError("input must be a JSON object mapping PR numbers to path arrays")
    result: dict[int, list[str]] = {}
    for key, value in raw.items():
        if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
            raise ValueError(f"PR {key} paths must be an array of strings")
        result[int(key)] = value
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="JSON mapping PR numbers to changed paths")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    report = analyze_pr_overlaps(_load(args.input))
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
