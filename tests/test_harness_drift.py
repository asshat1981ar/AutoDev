import importlib.util
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "check_harness_drift.py"
SPEC = importlib.util.spec_from_file_location("check_harness_drift", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
HARNESS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HARNESS)


class PlansContractTests(unittest.TestCase):
    def test_missing_required_fragment_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            plans = Path(tmp) / "PLANS.md"
            plans.write_text("# AutoDev ExecPlans\n", encoding="utf-8")
            errors: list[str] = []

            HARNESS.check_plans_contract(errors, False, plans)

            self.assertTrue(any("PLANS.md drift" in error for error in errors))

    def test_real_plans_contract_passes(self) -> None:
        errors: list[str] = []

        HARNESS.check_plans_contract(errors, False)

        self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
