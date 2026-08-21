import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from scripts.mistral_connector_sync import (
    ManifestError,
    MistralConnectorClient,
    apply_plan,
    diff_tools,
    load_manifest,
    plan_reconciliation,
    sanitize,
    validate_manifest,
)


BASE = {
    "schema_version": 1,
    "key": "deepwiki",
    "name": "autodev_deepwiki",
    "kind": "mcp",
    "managed": True,
    "server": "https://mcp.deepwiki.com/mcp",
    "visibility": "private",
    "description": "Repository intelligence",
    "tool_policy": {"include": [], "exclude": []},
    "confirmation": {"required": []},
    "risk": "read_only",
}


class ManifestTests(unittest.TestCase):
    def test_loads_json_subset_yaml_and_validates(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "connector.yaml"
            path.write_text(json.dumps(BASE), encoding="utf-8")
            self.assertEqual(load_manifest(path)["name"], "autodev_deepwiki")

    def test_rejects_invalid_name_visibility_and_secret_fields(self):
        for patch in (
            {"name": "bad name"},
            {"visibility": "world"},
            {"api_key": "secret"},
        ):
            with self.subTest(patch=patch), self.assertRaises(ManifestError):
                validate_manifest({**BASE, **patch})


class PlanningTests(unittest.TestCase):
    def test_create_noop_update_and_external(self):
        self.assertEqual(plan_reconciliation(BASE, None)["action"], "CREATE")
        remote = {
            "id": "uuid-1",
            "name": BASE["name"],
            "server": BASE["server"],
            "visibility": BASE["visibility"],
            "description": BASE["description"],
        }
        self.assertEqual(plan_reconciliation(BASE, remote)["action"], "NOOP")
        changed = {**remote, "description": "old"}
        update = plan_reconciliation(BASE, changed)
        self.assertEqual(update["action"], "UPDATE")
        self.assertEqual(update["connector_id"], "uuid-1")
        featured = {**BASE, "kind": "featured", "managed": False}
        self.assertEqual(plan_reconciliation(featured, None)["action"], "EXTERNAL")

    def test_shared_org_is_blocked_without_explicit_elevation(self):
        desired = {**BASE, "visibility": "shared_org"}
        self.assertEqual(plan_reconciliation(desired, None)["action"], "BLOCKED")
        self.assertEqual(
            plan_reconciliation(desired, None, allow_org_shared=True)["action"],
            "CREATE",
        )

    def test_tool_drift_and_redaction(self):
        old = [
            {"name": "read", "description": "old", "inputSchema": {"type": "object"}},
            {"name": "removed", "description": "x", "inputSchema": {}},
        ]
        new = [
            {"name": "read", "description": "new", "inputSchema": {"type": "object"}},
            {"name": "added", "description": "x", "inputSchema": {}},
        ]
        drift = diff_tools(old, new)
        self.assertEqual(drift["added"], ["added"])
        self.assertEqual(drift["removed"], ["removed"])
        self.assertEqual(drift["changed"], ["read"])
        clean = sanitize({"Authorization": "Bearer abc", "token": "abc", "ok": "value"})
        self.assertEqual(clean["Authorization"], "[REDACTED]")
        self.assertEqual(clean["token"], "[REDACTED]")
        self.assertEqual(clean["ok"], "value")


class FakeTransport:
    def __init__(self, responses):
        self.responses = list(responses)
        self.calls = []

    def __call__(self, method, url, headers, body):
        self.calls.append((method, url, headers, body))
        return self.responses.pop(0)


class ClientTests(unittest.TestCase):
    def test_client_serializes_documented_create_and_list_tools(self):
        transport = FakeTransport([
            {"id": "uuid-1", "name": BASE["name"]},
            [{"name": "read", "description": "Read", "inputSchema": {}}],
        ])
        client = MistralConnectorClient("test-key", transport=transport)
        client.create_connector(BASE)
        client.list_tools(BASE["name"], refresh=True, pretty=True)
        method, url, headers, body = transport.calls[0]
        self.assertEqual(method, "POST")
        self.assertTrue(url.endswith("/v1/connectors"))
        self.assertEqual(headers["Authorization"], "Bearer test-key")
        self.assertNotIn("risk", json.loads(body.decode("utf-8")))
        self.assertIn("refresh=true", transport.calls[1][1])
        self.assertIn("pretty=true", transport.calls[1][1])

    def test_apply_only_mutates_create_or_update(self):
        transport = FakeTransport([{"id": "uuid-1", "name": BASE["name"]}])
        client = MistralConnectorClient("test-key", transport=transport)
        result = apply_plan(client, {"action": "NOOP", "desired": BASE})
        self.assertEqual(result["action"], "NOOP")
        self.assertEqual(transport.calls, [])
        with self.assertRaises(ValueError):
            apply_plan(client, {"action": "BLOCKED", "desired": BASE})


if __name__ == "__main__":
    unittest.main()
