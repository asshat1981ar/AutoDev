import tempfile, unittest, zipfile
from pathlib import Path
from scripts.build_release import include

class TestReleaseBuilder(unittest.TestCase):
    def test_transients_are_excluded(self):
        self.assertFalse(include(Path("__pycache__/x.pyc")))
        self.assertFalse(include(Path(".worktrees/T/x.py")))
        self.assertFalse(include(Path("dist/release.zip")))
        self.assertTrue(include(Path("scripts/run_verification.sh")))
        self.assertTrue(include(Path("src/core/amx_dag.py")))
