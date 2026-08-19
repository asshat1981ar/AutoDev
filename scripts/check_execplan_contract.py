#!/usr/bin/env python3
"""Validate AutoDev's durable ExecPlan repository contract."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
PLANS = ROOT / "PLANS.md"
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


def validate() -> list[str]:
    if not PLANS.is_file():
        return ["PLANS.md drift: missing PLANS.md"]
    text = PLANS.read_text(encoding="utf-8")
    return [f"PLANS.md drift: missing fragment {fragment!r}" for fragment in REQUIRED_PLAN_FRAGMENTS if fragment not in text]


def main() -> int:
    errors = validate()
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("ExecPlan contract: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
