import json, tempfile, unittest
from argparse import Namespace
from pathlib import Path
from scripts.autodev_checkpoint import init

class TestAutoDevCheckpoint(unittest.TestCase):
    def args(self, out, **kw):
        d=dict(output=str(out), task_id="T-001", plan="docs/plan.md", revision="abc123",
               branch="feat/t1", worktree=".worktrees/T-001", next_action="write failing test")
        d.update(kw); return Namespace(**d)

    def test_init_writes_required_atomic_checkpoint(self):
        with tempfile.TemporaryDirectory() as td:
            out=Path(td)/".autodev/checkpoints/T-001.json"
            init(self.args(out))
            data=json.loads(out.read_text())
            required={"schema_version","task_id","plan_path","repository_revision","branch","worktree_path",
                      "completed_tasks","current_task","verification","effect_receipts","rulings","unresolved",
                      "next_action","updated_at"}
            self.assertTrue(required <= data.keys())
            self.assertFalse(out.with_suffix(".json.tmp").exists())

    def test_rejects_secret_like_checkpoint_value(self):
        with tempfile.TemporaryDirectory() as td:
            with self.assertRaises(ValueError):
                init(self.args(Path(td)/"x.json", next_action="Bearer abcdefghijklmnopqrstuvwxyz123456"))

    def test_rejects_external_absolute_worktree(self):
        with tempfile.TemporaryDirectory() as td:
            with self.assertRaises(ValueError):
                init(self.args(Path(td)/"x.json", worktree="/tmp/outside"))
