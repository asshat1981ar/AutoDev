import json
import unittest
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
PUBLIC_PROTOCOL = ROOT / "protocols" / "public" / "v1"
FIXTURES = PUBLIC_PROTOCOL / "fixtures"

REQUIRED_SCHEMAS = {
    "objective-summary.schema.json",
    "objective-create.schema.json",
    "objective-event.schema.json",
    "evidence-summary.schema.json",
    "code-graph-snapshot.schema.json",
    "connectivity-status.schema.json",
    "protocol-error.schema.json",
}

REQUIRED_FIXTURES = {
    "objective-summary.queued.json",
    "objective-create.json",
    "objective-event.queued.json",
    "evidence-summary.passed.json",
    "code-graph-snapshot.json",
    "connectivity-status.ready.json",
    "protocol-error.json",
}

FORBIDDEN_AUTHORITY_KEYS = {
    "approval_ref",
    "authorization",
    "authorization_grant",
    "capabilities",
    "policy",
    "task_graph",
}


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def collect_keys(value: Any) -> set[str]:
    keys: set[str] = set()
    if isinstance(value, dict):
        for key, nested in value.items():
            keys.add(str(key))
            keys.update(collect_keys(nested))
    elif isinstance(value, list):
        for nested in value:
            keys.update(collect_keys(nested))
    return keys


class PublicProtocolTests(unittest.TestCase):
    def test_required_public_v1_contracts_exist(self) -> None:
        schema_names = {path.name for path in PUBLIC_PROTOCOL.glob("*.schema.json")}
        fixture_names = {path.name for path in FIXTURES.glob("*.json")}
        self.assertTrue(REQUIRED_SCHEMAS.issubset(schema_names))
        self.assertTrue(REQUIRED_FIXTURES.issubset(fixture_names))

    def test_every_public_protocol_json_document_parses(self) -> None:
        documents = sorted(PUBLIC_PROTOCOL.rglob("*.json"))
        self.assertGreater(len(documents), 0)
        for path in documents:
            with self.subTest(path=path.relative_to(ROOT)):
                self.assertIsNotNone(load_json(path))

    def test_public_fixtures_do_not_carry_trusted_authority_fields(self) -> None:
        for path in sorted(FIXTURES.glob("*.json")):
            with self.subTest(path=path.name):
                keys = collect_keys(load_json(path))
                self.assertTrue(FORBIDDEN_AUTHORITY_KEYS.isdisjoint(keys), sorted(keys))

    def test_versioned_read_fixtures_use_public_schema_version_one(self) -> None:
        for path in sorted(FIXTURES.glob("*.json")):
            if path.name == "objective-create.json":
                continue
            with self.subTest(path=path.name):
                payload = load_json(path)
                self.assertEqual(payload.get("schema_version"), "1")


if __name__ == "__main__":
    unittest.main()
