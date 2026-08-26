import importlib.util
import json
import subprocess
import sys
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "scripts/autodev-cli.py"
WEB_DIR = ROOT / "web/command-center"


def load_cli_module():
    spec = importlib.util.spec_from_file_location("autodev_cli", CLI)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to load AutoDev CLI module")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class FakeAutoDevHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    objectives = [
        {
            "id": "objective-1",
            "repository": "owner/repo",
            "description": "Inspect current state",
            "branch": "autodev/objective-1",
            "status": "queued",
            "graph": {"root": {"description": "Inspect current state"}},
        }
    ]

    def log_message(self, format, *args):
        return

    def _json(self, status, payload):
        encoded = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)

    def do_GET(self):
        if self.path == "/api/v1/objectives":
            self._json(200, self.objectives)
            return
        self._json(404, {"error": "not found"})

    def do_POST(self):
        if self.path != "/api/v1/objectives":
            self._json(404, {"error": "not found"})
            return
        length = int(self.headers.get("content-length", "0"))
        request = json.loads(self.rfile.read(length).decode("utf-8"))
        created = {
            "id": "objective-created",
            "repository": request["repository"],
            "description": request["description"],
            "branch": request.get("branch") or "autodev/objective-created",
            "status": "queued",
            "graph": {"root": {"description": request["description"]}},
        }
        self._json(202, created)


class FakeSseResponse:
    status = 200

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        return False

    def __iter__(self):
        return iter([b"data: {\"type\":\"objective_queued\"}\n"])


class CommandCenterClientTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), FakeAutoDevHandler)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        cls.base_url = f"http://127.0.0.1:{cls.server.server_port}"

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join(timeout=2)

    def run_cli(self, *args):
        env = {
            **__import__("os").environ,
            "NO_PROXY": "127.0.0.1,localhost",
            "no_proxy": "127.0.0.1,localhost",
        }
        return subprocess.run(
            [sys.executable, str(CLI), "--server", self.base_url, *args],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
            env=env,
        )

    def test_cli_lists_objectives_as_json(self):
        result = self.run_cli("objectives", "list", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload[0]["id"], "objective-1")
        self.assertEqual(payload[0]["status"], "queued")

    def test_cli_creates_objective_without_execution_authority(self):
        result = self.run_cli(
            "objectives",
            "create",
            "--repository",
            "owner/repo",
            "--description",
            "Add a bounded vertical slice",
            "--json",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["id"], "objective-created")
        self.assertEqual(payload["status"], "queued")
        self.assertNotIn("approved", payload)
        self.assertNotIn("execute", payload)

    def test_cli_rejects_non_http_server_urls(self):
        env = {
            **__import__("os").environ,
            "NO_PROXY": "127.0.0.1,localhost",
            "no_proxy": "127.0.0.1,localhost",
        }
        result = subprocess.run(
            [sys.executable, str(CLI), "--server", "file:///tmp", "objectives", "list"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
            env=env,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("http:// or https://", result.stderr)

    def test_cli_event_stream_has_no_read_timeout(self):
        module = load_cli_module()
        client = module.Client("http://127.0.0.1:8080", timeout=0.1)
        with patch.object(module, "urlopen", return_value=FakeSseResponse()) as opener:
            lines = list(client.event_lines("/events"))

        self.assertEqual(lines, ['{"type":"objective_queued"}'])
        self.assertEqual(opener.call_count, 1)
        _, kwargs = opener.call_args
        self.assertIsNone(kwargs.get("timeout"))

    def test_web_client_is_framework_free_and_uses_existing_http_sse_contract(self):
        html = (WEB_DIR / "index.html").read_text(encoding="utf-8")
        javascript = (WEB_DIR / "app.js").read_text(encoding="utf-8")

        self.assertIn("AutoDev Command Center", html)
        self.assertIn("/api/v1/objectives", javascript)
        self.assertIn("/events", javascript)
        self.assertIn("EventSource", javascript)
        self.assertNotIn("/mcp", javascript)
        self.assertNotIn("node_modules", html)

        syntax = subprocess.run(
            ["node", "--check", str(WEB_DIR / "app.js")],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(syntax.returncode, 0, syntax.stderr)


if __name__ == "__main__":
    unittest.main()
