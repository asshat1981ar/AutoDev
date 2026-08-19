import unittest
from pathlib import Path


class ExecPlanContractTests(unittest.TestCase):
    def test_required_contract_fragments_are_present(self):
        root = Path(__file__).resolve().parents[1]
        text = (root / "PLANS.md").read_text(encoding="utf-8")
        required = [
            "## Non-negotiable authority boundary",
            "### Progress",
            "### Surprises & Discoveries",
            "### Decision Log",
            "### Outcomes & Retrospective",
            "An ExecPlan is durable coordination state, not execution authority.",
            "reconciliation",
            "verification",
        ]
        missing = [fragment for fragment in required if fragment not in text]
        self.assertEqual([], missing, f"PLANS.md drift: missing {missing}")


if __name__ == "__main__":
    unittest.main()
