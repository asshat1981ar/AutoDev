import os
import tempfile
import unittest
import zipfile
from pathlib import Path

from scripts.build_release import include, should_package


class TestReleaseBuilder(unittest.TestCase):
    def test_transients_are_excluded(self):
        self.assertFalse(include(Path("__pycache__/x.pyc")))
        self.assertFalse(include(Path(".worktrees/T/x.py")))
        self.assertFalse(include(Path("dist/release.zip")))
        self.assertTrue(include(Path("scripts/run_verification.sh")))
        self.assertTrue(include(Path("src/core/amx_dag.py")))

    def test_symlinked_files_are_never_packaged(self):
        with tempfile.TemporaryDirectory() as td, tempfile.TemporaryDirectory() as external:
            root = Path(td)
            outside = Path(external) / "secret.txt"
            outside.write_text("must not escape source boundary")
            linked = root / "linked-secret.txt"
            try:
                linked.symlink_to(outside)
            except (OSError, NotImplementedError):
                self.skipTest("symlinks are unavailable on this platform")

            self.assertFalse(should_package(linked, root, root / "release.zip"))

    def test_output_and_checksum_sidecar_are_never_packaged(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            output = root / "release.zip"
            sidecar = Path(str(output) + ".sha256")
            ordinary = root / "README.md"
            output.write_bytes(b"old archive")
            sidecar.write_text("old checksum")
            ordinary.write_text("release input")

            self.assertFalse(should_package(output, root, output))
            self.assertFalse(should_package(sidecar, root, output))
            self.assertTrue(should_package(ordinary, root, output))
