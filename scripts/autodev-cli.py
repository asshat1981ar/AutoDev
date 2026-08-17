#!/usr/bin/env python3
"""Dependency-free CLI for AutoDev's existing HTTP/SSE control-plane API.

This client can observe objectives, enqueue new objectives, and follow event
streams. It intentionally exposes no ForgeCore execution, approval, Git, or MCP
authority.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from typing import Any, Iterable
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit, urlunsplit
from urllib.request import Request, urlopen

DEFAULT_SERVER = "http://127.0.0.1:8080"
DEFAULT_TIMEOUT_SECONDS = 10.0


class CliError(RuntimeError):
    """User-facing command failure."""


@dataclass(frozen=True)
class Client:
    base_url: str
    timeout: float

    def get_json(self, path: str) -> Any:
        request = Request(self._url(path), method="GET", headers={"accept": "application/json"})
        return self._json_request(request, expected_statuses={200})

    def post_json(self, path: str, payload: dict[str, Any]) -> Any:
        body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        request = Request(
            self._url(path),
            data=body,
            method="POST",
            headers={
                "accept": "application/json",
                "content-type": "application/json",
            },
        )
        return self._json_request(request, expected_statuses={200, 201, 202})

    def event_lines(self, path: str) -> Iterable[str]:
        request = Request(
            self._url(path),
            method="GET",
            headers={"accept": "text/event-stream"},
        )
        try:
            with urlopen(request, timeout=self.timeout) as response:
                if response.status != 200:
                    raise CliError(f"unexpected HTTP status {response.status}")
                for raw_line in response:
                    line = raw_line.decode("utf-8", errors="replace").rstrip("\r\n")
                    if line.startswith("data:"):
                        yield line[5:].lstrip()
        except HTTPError as error:
            raise CliError(_http_error_message(error)) from error
        except URLError as error:
            raise CliError(f"request failed: {error.reason}") from error

    def _json_request(self, request: Request, expected_statuses: set[int]) -> Any:
        try:
            with urlopen(request, timeout=self.timeout) as response:
                body = response.read()
                if response.status not in expected_statuses:
                    raise CliError(f"unexpected HTTP status {response.status}")
        except HTTPError as error:
            raise CliError(_http_error_message(error)) from error
        except URLError as error:
            raise CliError(f"request failed: {error.reason}") from error

        try:
            return json.loads(body.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise CliError("server returned invalid JSON") from error

    def _url(self, path: str) -> str:
        return f"{self.base_url}{path}"


def normalize_server(value: str) -> str:
    parsed = urlsplit(value.strip())
    if parsed.scheme not in {"http", "https"}:
        raise argparse.ArgumentTypeError("server URL must start with http:// or https://")
    if not parsed.hostname:
        raise argparse.ArgumentTypeError("server URL must include a host")
    if parsed.username is not None or parsed.password is not None:
        raise argparse.ArgumentTypeError("server URL must not embed credentials")
    if parsed.query or parsed.fragment:
        raise argparse.ArgumentTypeError("server URL must not include query or fragment components")

    path = parsed.path.rstrip("/")
    return urlunsplit((parsed.scheme, parsed.netloc, path, "", ""))


def _http_error_message(error: HTTPError) -> str:
    try:
        body = error.read().decode("utf-8", errors="replace")
    except OSError:
        body = ""
    detail = body.strip()
    if detail:
        try:
            payload = json.loads(detail)
            if isinstance(payload, dict) and isinstance(payload.get("error"), str):
                detail = payload["error"]
        except json.JSONDecodeError:
            pass
    suffix = f": {detail}" if detail else ""
    return f"HTTP {error.code}{suffix}"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="AutoDev command-center CLI")
    parser.add_argument(
        "--server",
        type=normalize_server,
        default=normalize_server(DEFAULT_SERVER),
        help=f"AutoDev server base URL (default: {DEFAULT_SERVER})",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=DEFAULT_TIMEOUT_SECONDS,
        help=f"HTTP timeout in seconds (default: {DEFAULT_TIMEOUT_SECONDS:g})",
    )

    commands = parser.add_subparsers(dest="command", required=True)

    objectives = commands.add_parser("objectives", help="Inspect or enqueue objectives")
    objective_commands = objectives.add_subparsers(dest="objective_command", required=True)

    list_parser = objective_commands.add_parser("list", help="List current objectives")
    list_parser.add_argument("--json", action="store_true", help="Emit JSON")

    create_parser = objective_commands.add_parser("create", help="Enqueue a new objective")
    create_parser.add_argument("--repository", required=True, help="Repository identifier")
    create_parser.add_argument("--description", required=True, help="Objective description")
    create_parser.add_argument("--branch", help="Optional target branch")
    create_parser.add_argument("--json", action="store_true", help="Emit JSON")

    events = commands.add_parser("events", help="Follow the server SSE event stream")
    events.add_argument("--json", action="store_true", help="Normalize each event as JSON")

    return parser


def render_objectives(payload: Any) -> str:
    if not isinstance(payload, list):
        raise CliError("objective list response is not an array")
    if not payload:
        return "No objectives."

    rows = []
    for item in payload:
        if not isinstance(item, dict):
            raise CliError("objective list contains a non-object value")
        rows.append(
            (
                str(item.get("id", "")),
                str(item.get("status", "")),
                str(item.get("repository", "")),
                str(item.get("branch", "")),
                str(item.get("description", "")),
            )
        )

    widths = [max(len(row[index]) for row in rows) for index in range(4)]
    header = ("ID", "STATUS", "REPOSITORY", "BRANCH")
    widths = [max(widths[index], len(header[index])) for index in range(4)]
    lines = [
        f"{header[0]:<{widths[0]}}  {header[1]:<{widths[1]}}  "
        f"{header[2]:<{widths[2]}}  {header[3]:<{widths[3]}}  DESCRIPTION"
    ]
    for row in rows:
        lines.append(
            f"{row[0]:<{widths[0]}}  {row[1]:<{widths[1]}}  "
            f"{row[2]:<{widths[2]}}  {row[3]:<{widths[3]}}  {row[4]}"
        )
    return "\n".join(lines)


def render_created(payload: Any) -> str:
    if not isinstance(payload, dict):
        raise CliError("objective creation response is not an object")
    return "\n".join(
        [
            f"id: {payload.get('id', '')}",
            f"status: {payload.get('status', '')}",
            f"repository: {payload.get('repository', '')}",
            f"branch: {payload.get('branch', '')}",
            f"description: {payload.get('description', '')}",
        ]
    )


def json_output(payload: Any) -> str:
    return json.dumps(payload, indent=2, sort_keys=True)


def run(args: argparse.Namespace) -> int:
    if args.timeout <= 0:
        raise CliError("timeout must be greater than zero")
    client = Client(args.server, args.timeout)

    if args.command == "objectives" and args.objective_command == "list":
        payload = client.get_json("/api/v1/objectives")
        print(json_output(payload) if args.json else render_objectives(payload))
        return 0

    if args.command == "objectives" and args.objective_command == "create":
        repository = args.repository.strip()
        description = args.description.strip()
        branch = args.branch.strip() if args.branch else None
        if not repository:
            raise CliError("repository must not be empty")
        if not description:
            raise CliError("description must not be empty")
        payload: dict[str, Any] = {
            "repository": repository,
            "description": description,
        }
        if branch:
            payload["branch"] = branch
        created = client.post_json("/api/v1/objectives", payload)
        print(json_output(created) if args.json else render_created(created))
        return 0

    if args.command == "events":
        for data in client.event_lines("/events"):
            if args.json:
                try:
                    print(json_output(json.loads(data)))
                except json.JSONDecodeError:
                    print(json_output({"data": data}))
            else:
                print(data)
        return 0

    raise CliError("unsupported command")


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return run(args)
    except CliError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
