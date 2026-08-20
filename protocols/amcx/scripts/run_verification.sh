#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
export PYTHONPATH="${REPO_ROOT}/src${PYTHONPATH:+:${PYTHONPATH}}"
cd "${REPO_ROOT}"

echo "========================================================"
echo " AMCX-1 / AUTODEV COMPREHENSIVE VERIFICATION SUITE      "
echo "========================================================"

python3 - <<'PY'
import sys, unittest
suite = unittest.defaultTestLoader.discover("tests", pattern="test_*.py")
count = suite.countTestCases()
result = unittest.TextTestRunner(verbosity=2).run(suite)
print("========================================================")
if result.wasSuccessful():
    print(f" ALL TESTS PASSED CLEANLY ({count}/{count} VERIFIED)")
else:
    passed = count - len(result.failures) - len(result.errors)
    print(f" VERIFICATION FAILED ({passed}/{count} passed; {len(result.failures)} failures; {len(result.errors)} errors)")
print("========================================================")
sys.exit(0 if result.wasSuccessful() else 1)
PY
