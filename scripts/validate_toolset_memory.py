#!/usr/bin/env python3
"""Validate AutoDev's repository-local toolset learning memory."""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import date
from pathlib import Path
from typing import Any

REQUIRED_FIELDS = (
    "schema_version",
    "pattern_id",
    "task_class",
    "context",
    "combination",
    "result",
    "evidence",
    "strengths",
    "failure_modes",
    "when_to_use",
    "when_not_to_use",
    "confidence",
    "sample_size",
    "last_validated",
)

PATTERN_ID_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
VALID_RESULTS = {"high", "medium", "low", "failed"}
VALID_CONFIDENCE = {"high", "medium", "low"}
LIST_FIELDS = (
    "combination",
    "evidence",
    "strengths",
    "failure_modes",
    "when_to_use",
    "when_not_to_use",
)


def _nonempty_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _string_list(value: Any, *, require_nonempty: bool = True) -> bool:
    if not isinstance(value, list):
        return False
    if require_nonempty and not value:
        return False
    return all(_nonempty_string(item) for item in value)


def validate_record(record: dict[str, Any]) -> list[str]:
    errors: list[str] = []

    for field in REQUIRED_FIELDS:
        if field not in record:
            errors.append(f"missing required field: {field}")

    if errors:
        return errors

    if record["schema_version"] != "toolset-pattern-v1":
        errors.append("schema_version must be toolset-pattern-v1")

    pattern_id = record["pattern_id"]
    if not _nonempty_string(pattern_id) or not PATTERN_ID_RE.fullmatch(pattern_id):
        errors.append("pattern_id must be a lowercase hyphen-separated slug")

    for field in ("task_class", "context"):
        if not _nonempty_string(record[field]):
            errors.append(f"{field} must be a non-empty string")

    for field in LIST_FIELDS:
        if not _string_list(record[field]):
            errors.append(f"{field} must be a non-empty list of non-empty strings")

    if record["result"] not in VALID_RESULTS:
        errors.append(f"result must be one of {sorted(VALID_RESULTS)}")

    if record["confidence"] not in VALID_CONFIDENCE:
        errors.append(f"confidence must be one of {sorted(VALID_CONFIDENCE)}")

    sample_size = record["sample_size"]
    if isinstance(sample_size, bool) or not isinstance(sample_size, int) or sample_size <= 0:
        errors.append("sample_size must be a positive integer")

    last_validated = record["last_validated"]
    if not _nonempty_string(last_validated):
        errors.append("last_validated must be an ISO YYYY-MM-DD date")
    else:
        try:
            parsed = date.fromisoformat(last_validated)
        except ValueError:
            errors.append("last_validated must be an ISO YYYY-MM-DD date")
        else:
            if parsed.isoformat() != last_validated:
                errors.append("last_validated must be an ISO YYYY-MM-DD date")

    return errors


def validate_file(path: Path) -> list[str]:
    errors: list[str] = []
    seen_ids: set[str] = set()

    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        return [f"{path}: unable to read file: {exc}"]

    for line_number, raw_line in enumerate(lines, start=1):
        if not raw_line.strip():
            continue

        try:
            value = json.loads(raw_line)
        except json.JSONDecodeError as exc:
            errors.append(f"{path}:{line_number}: invalid JSON: {exc.msg}")
            continue

        if not isinstance(value, dict):
            errors.append(f"{path}:{line_number}: record must be a JSON object")
            continue

        record_errors = validate_record(value)
        errors.extend(f"{path}:{line_number}: {error}" for error in record_errors)

        pattern_id = value.get("pattern_id")
        if isinstance(pattern_id, str):
            if pattern_id in seen_ids:
                errors.append(f"{path}:{line_number}: duplicate pattern_id: {pattern_id}")
            seen_ids.add(pattern_id)

    if not any(line.strip() for line in lines):
        errors.append(f"{path}: dataset must contain at least one record")

    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", type=Path, help="JSONL dataset to validate")
    args = parser.parse_args(argv)

    errors = validate_file(args.path)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1

    record_count = sum(
        1
        for line in args.path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    )
    print(f"validated {record_count} toolset pattern records")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
