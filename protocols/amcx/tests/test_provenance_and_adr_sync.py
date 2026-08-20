import unittest
import os
import re

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

class TestProvenanceAndAdrSync(unittest.TestCase):
    def test_adr004_has_all_18_domains(self):
        adr_path = str(REPO_ROOT / 'docs' / 'ADRs' / 'ADR-004-source-of-truth-and-trust-boundaries.md')
        self.assertTrue(os.path.exists(adr_path))
        with open(adr_path) as f:
            content = f.read()

        expected_domains = [
            "Plan and step lifecycle",
            "Collaboration",
            "Portable memory",
            "Origin/receiver identity",
            "Evidence verdict/freshness",
            "Quarantine restriction",
            "Release/trust/visibility widening",
            "Retraction suppression barriers",
            "Cross-project grant",
            "Effective retrieval",
            "Effects and receipts",
            "ContextView history",
            "Hard purge",
            "Prompt/skill/router activation",
            "GateProfile publication/status",
            "Contract activation",
            "Artifact bytes",
            "Aggregate budgets"
        ]
        for d in expected_domains:
            self.assertIn(d, content, f"Domain '{d}' missing from ADR-004!")

    def test_no_raw_secret_leak_in_specs_or_schemas(self):
        repo_dir = str(REPO_ROOT / 'registry' / 'v1')
        secret_regex = re.compile(r"(?i)(api_key|bearer\s+[a-z0-9_\-\.]{20,}|ghp_[a-zA-Z0-9]{36}|sk-[a-zA-Z0-9]{20,})")
        for fname in os.listdir(repo_dir):
            if fname.endswith('.json'):
                with open(os.path.join(repo_dir, fname)) as f:
                    data = f.read()
                    self.assertIsNone(secret_regex.search(data), f"Secret pattern matched in schema {fname}!")

if __name__ == '__main__':
    unittest.main()
