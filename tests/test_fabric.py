import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import install

ROOT = Path(__file__).resolve().parents[1]


class FabricTests(unittest.TestCase):
    def test_package_configuration_is_valid(self):
        for relative in (
            ".cline/config/capabilities.json",
            ".cline/config/permissions.json",
            ".cline/hooks/hooks.json",
            ".cline/plugins/project-fabric/plugin.json",
        ):
            with self.subTest(relative=relative):
                json.loads((ROOT / relative).read_text(encoding="utf-8"))

    def test_dry_run_does_not_write_project(self):
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(
                [sys.executable, str(ROOT / "install.py"), "--project", directory, "--dry-run"],
                cwd=ROOT, capture_output=True, text=True, check=True,
            )
            self.assertIn("WRITE", result.stdout)
            self.assertEqual(list(Path(directory).iterdir()), [])

    def test_manifests_reference_existing_runtime_files(self):
        install.validate_package()
        hooks = json.loads((ROOT / ".cline/hooks/hooks.json").read_text(encoding="utf-8"))
        for script in hooks.values():
            self.assertTrue((ROOT / ".cline/hooks" / script).is_file())
        plugin = json.loads((ROOT / ".cline/plugins/project-fabric/plugin.json").read_text(encoding="utf-8"))
        self.assertTrue((ROOT / ".cline/plugins/project-fabric" / plugin["entry"]).is_file())

    def test_install_marks_hook_scripts_executable(self):
        with tempfile.TemporaryDirectory() as directory:
            subprocess.run(
                [sys.executable, str(ROOT / "install.py"), "--project", directory],
                cwd=ROOT, capture_output=True, text=True, check=True,
            )
            hook = Path(directory) / ".cline/hooks/pre_tool_use.py"
            self.assertTrue(hook.stat().st_mode & 0o111)

    def test_destructive_hook_blocks_command(self):
        hook = ROOT / ".cline/hooks/pre_tool_use.py"
        result = subprocess.run(
            [sys.executable, str(hook)],
            input=json.dumps({"tool_input": {"command": "git reset --hard HEAD"}}),
            text=True, capture_output=True,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn('"cancel": true', result.stdout)

    def test_destructive_hook_blocks_compact_force_variants(self):
        hook = ROOT / ".cline/hooks/pre_tool_use.py"
        for command in [
            "rm -fr build",
            "rm -r -f build",
            "rm --recursive --force build",
            "git push -f origin main",
            "git push --force origin main",
        ]:
            result = subprocess.run(
                [sys.executable, str(hook)],
                input=json.dumps({"tool_input": {"command": command}}),
                text=True, capture_output=True,
            )
            self.assertEqual(
                result.returncode,
                2,
                f"expected destructive command to be blocked: {command!r}",
            )
            self.assertIn('"cancel": true', result.stdout)

    def test_destructive_hook_allows_force_with_lease(self):
        # The safer `--force-with-lease` form must NOT be blocked by the
        # `--force` clause (negative lookahead in the regex).
        hook = ROOT / ".cline/hooks/pre_tool_use.py"
        result = subprocess.run(
            [sys.executable, str(hook)],
            input=json.dumps({"tool_input": {"command": "git push --force-with-lease origin main"}}),
            text=True, capture_output=True,
        )
        self.assertEqual(result.returncode, 0)
        self.assertNotIn('"cancel": true', result.stdout)


if __name__ == "__main__":
    unittest.main()