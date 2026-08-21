#!/usr/bin/env python3
"""Git-authoritative Mistral Studio Connector reconciliation.

Connector manifests are strict JSON documents stored with ``.yaml`` suffixes.
JSON is a YAML subset, keeping this AutoDev integration dependency-free.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path, PurePosixPath
from typing import Any, Callable
from urllib.parse import quote, urlencode
from urllib.request import Request, urlopen

API_BASE = "https://api.mistral.ai"
NAME_RE = re.compile(r"^[A-Za-z0-9_-]{1,64}$")
VISIBILITIES = {"private", "shared_workspace", "shared_org"}
KINDS = {"mcp", "featured"}
ACTIONS = {"CREATE", "UPDATE", "NOOP", "EXTERNAL", "BLOCKED"}
REMOTE_FIELDS = (
    "name",
    "description",
    "server",
    "visibility",
    "icon_url",
    "system_prompt",
)
SECRET_FRAGMENTS = (
    "authorization",
    "api_key",
    "apikey",
    "password",
    "secret",
    "token",
)


class ManifestError(ValueError):
    """Raised when desired Connector state is unsafe or malformed."""


def _normalized_key(key: str) -> str:
    return key.lower().replace("-", "_")


def _is_secret_key(key: str) -> bool:
    lowered = _normalized_key(key)
    if lowered.endswith("_ref") or lowered.endswith("_reference"):
        return False
    return any(fragment in lowered for fragment in SECRET_FRAGMENTS)


def _reject_inline_secrets(value: Any, path: str = "$") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if _is_secret_key(str(key)):
                raise ManifestError(f"secret-like field is forbidden at {path}.{key}")
            _reject_inline_secrets(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _reject_inline_secrets(child, f"{path}[{index}]")


def validate_manifest(data: dict[str, Any]) -> dict[str, Any]:
    if not isinstance(data, dict):
        raise ManifestError("manifest must be an object")
    _reject_inline_secrets(data)
    if data.get("schema_version") != 1:
        raise ManifestError("schema_version must be 1")
    for field in ("key", "name", "kind", "managed", "visibility"):
        if field not in data:
            raise ManifestError(f"missing required field: {field}")
    if not isinstance(data["key"], str) or not data["key"]:
        raise ManifestError("key must be a non-empty string")
    if not isinstance(data["name"], str) or not NAME_RE.fullmatch(data["name"]):
        raise ManifestError("name must be 1-64 characters: alphanumeric, '_' or '-'")
    if data["kind"] not in KINDS:
        raise ManifestError(f"kind must be one of {sorted(KINDS)}")
    if not isinstance(data["managed"], bool):
        raise ManifestError("managed must be boolean")
    if data["visibility"] not in VISIBILITIES:
        raise ManifestError(f"visibility must be one of {sorted(VISIBILITIES)}")
    if data["kind"] == "mcp":
        server = data.get("server")
        if not isinstance(server, str) or not server.startswith("https://"):
            raise ManifestError("managed MCP connector server must be an https URL")
    if data["kind"] == "featured" and data["managed"]:
        raise ManifestError("featured connectors are external and must set managed=false")
    for container in ("tool_policy", "confirmation"):
        if container in data and not isinstance(data[container], dict):
            raise ManifestError(f"{container} must be an object")
    return data


def validate_registry(data: dict[str, Any]) -> dict[str, Any]:
    """Validate the dependency-free registry that enumerates desired manifests."""
    if not isinstance(data, dict):
        raise ManifestError("registry must be an object")
    _reject_inline_secrets(data)
    if data.get("schema_version") != 1:
        raise ManifestError("registry schema_version must be 1")
    connectors = data.get("connectors")
    if not isinstance(connectors, list) or not connectors:
        raise ManifestError("registry connectors must be a non-empty list")
    if len(connectors) != len(set(connectors)):
        raise ManifestError("registry connector paths must be unique")
    for item in connectors:
        if not isinstance(item, str) or not item:
            raise ManifestError("registry connector paths must be non-empty strings")
        path = PurePosixPath(item)
        if path.is_absolute() or ".." in path.parts:
            raise ManifestError(f"unsafe registry connector path: {item}")
        if len(path.parts) < 2 or path.parts[0] != "connectors" or path.suffix != ".yaml":
            raise ManifestError(f"registry connector path must be connectors/*.yaml: {item}")
    return data


def _read_json(path: str | Path) -> Any:
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ManifestError(f"cannot load manifest {path}: {exc}") from exc


def load_manifest(path: str | Path) -> dict[str, Any]:
    return validate_manifest(_read_json(path))


def load_validation_target(path: str | Path) -> dict[str, Any]:
    data = _read_json(path)
    if isinstance(data, dict) and "connectors" in data and "key" not in data:
        return validate_registry(data)
    return validate_manifest(data)


def sanitize(value: Any) -> Any:
    if isinstance(value, dict):
        return {
            key: "[REDACTED]" if _is_secret_key(str(key)) else sanitize(child)
            for key, child in value.items()
        }
    if isinstance(value, list):
        return [sanitize(item) for item in value]
    return value


def _comparable(desired: dict[str, Any]) -> dict[str, Any]:
    return {field: desired[field] for field in REMOTE_FIELDS if field in desired}


def plan_reconciliation(
    desired: dict[str, Any],
    remote: dict[str, Any] | None,
    *,
    allow_org_shared: bool = False,
) -> dict[str, Any]:
    desired = validate_manifest(dict(desired))
    if desired["kind"] == "featured" or not desired["managed"]:
        return {
            "action": "EXTERNAL",
            "desired": desired,
            "reason": "resource is not managed",
        }
    if desired["visibility"] == "shared_org" and not allow_org_shared:
        return {
            "action": "BLOCKED",
            "desired": desired,
            "reason": "shared_org mutation requires explicit elevation",
        }
    if remote is None:
        return {"action": "CREATE", "desired": desired}

    changes = {
        field: expected
        for field, expected in _comparable(desired).items()
        if remote.get(field) != expected
    }
    if not changes:
        return {
            "action": "NOOP",
            "desired": desired,
            "connector_id": remote.get("id"),
        }
    if "visibility" in changes:
        return {
            "action": "BLOCKED",
            "desired": desired,
            "connector_id": remote.get("id"),
            "changes": changes,
            "reason": (
                "visibility drift cannot be updated in place by the documented "
                "Connector update API"
            ),
        }
    connector_id = remote.get("id")
    if not connector_id:
        return {
            "action": "BLOCKED",
            "desired": desired,
            "changes": changes,
            "reason": "remote connector update requires UUID",
        }
    return {
        "action": "UPDATE",
        "desired": desired,
        "connector_id": str(connector_id),
        "changes": changes,
    }


def _tool_map(tools: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    mapped: dict[str, dict[str, Any]] = {}
    for tool in tools:
        name = tool.get("name")
        if isinstance(name, str) and name:
            mapped[name] = sanitize(tool)
    return mapped


def diff_tools(
    previous: list[dict[str, Any]], current: list[dict[str, Any]]
) -> dict[str, list[str]]:
    old = _tool_map(previous)
    new = _tool_map(current)
    old_names = set(old)
    new_names = set(new)
    return {
        "added": sorted(new_names - old_names),
        "removed": sorted(old_names - new_names),
        "changed": sorted(name for name in old_names & new_names if old[name] != new[name]),
    }


Transport = Callable[[str, str, dict[str, str], bytes | None], Any]


def _default_transport(
    method: str,
    url: str,
    headers: dict[str, str],
    body: bytes | None,
) -> Any:
    request = Request(url=url, data=body, method=method, headers=headers)
    with urlopen(request, timeout=30) as response:  # nosec B310 - fixed API base by CLI
        payload = response.read()
    return json.loads(payload.decode("utf-8")) if payload else {}


class MistralConnectorClient:
    """Small REST adapter around the documented Connector endpoints."""

    def __init__(
        self,
        api_key: str,
        *,
        base_url: str = API_BASE,
        transport: Transport | None = None,
    ) -> None:
        if not api_key:
            raise ValueError("api_key is required for live Mistral calls")
        self._api_key = api_key
        self._base_url = base_url.rstrip("/")
        self._transport = transport or _default_transport

    def _request(
        self,
        method: str,
        path: str,
        payload: dict[str, Any] | None = None,
    ) -> Any:
        headers = {
            "Authorization": f"Bearer {self._api_key}",
            "Accept": "application/json",
        }
        body = None
        if payload is not None:
            headers["Content-Type"] = "application/json"
            body = json.dumps(
                payload, sort_keys=True, separators=(",", ":")
            ).encode("utf-8")
        return self._transport(method, f"{self._base_url}{path}", headers, body)

    def list_connectors(self, *, page_size: int = 100) -> list[dict[str, Any]]:
        items: list[dict[str, Any]] = []
        cursor: str | None = None
        while True:
            params: dict[str, str] = {"page_size": str(page_size)}
            if cursor:
                params["cursor"] = cursor
            response = self._request("GET", f"/v1/connectors?{urlencode(params)}")
            if isinstance(response, list):
                items.extend(response)
                return items
            page_items = response.get("items", response.get("data", []))
            if not isinstance(page_items, list):
                raise ValueError("unexpected connector list response")
            items.extend(page_items)
            pagination = response.get("pagination") or {}
            cursor = pagination.get("next_cursor") or response.get("next_cursor")
            if not cursor:
                return items

    def get_connector(self, connector_id_or_name: str) -> dict[str, Any]:
        return self._request(
            "GET", f"/v1/connectors/{quote(connector_id_or_name, safe='')}"
        )

    def create_connector(self, desired: dict[str, Any]) -> dict[str, Any]:
        validate_manifest(desired)
        if desired["kind"] != "mcp" or not desired["managed"]:
            raise ValueError("only managed MCP connectors can be created")
        allowed = (
            "name",
            "description",
            "server",
            "visibility",
            "icon_url",
            "system_prompt",
        )
        payload = {field: desired[field] for field in allowed if field in desired}
        return self._request("POST", "/v1/connectors", payload)

    def update_connector(
        self, connector_id: str, changes: dict[str, Any]
    ) -> dict[str, Any]:
        allowed = {"name", "description", "server", "icon_url", "system_prompt"}
        unsupported = set(changes) - allowed
        if unsupported:
            raise ValueError(
                f"unsupported connector update fields: {sorted(unsupported)}"
            )
        return self._request(
            "PATCH",
            f"/v1/connectors/{quote(connector_id, safe='')}",
            changes,
        )

    def list_tools(
        self,
        connector_id_or_name: str,
        *,
        refresh: bool = False,
        pretty: bool = True,
    ) -> list[dict[str, Any]]:
        params = urlencode(
            {
                "refresh": "true" if refresh else "false",
                "pretty": "true" if pretty else "false",
                "page_size": "100",
            }
        )
        response = self._request(
            "GET",
            f"/v1/connectors/{quote(connector_id_or_name, safe='')}/tools?{params}",
        )
        if isinstance(response, list):
            return response
        tools = response.get("items", response.get("data", response.get("tools", [])))
        if not isinstance(tools, list):
            raise ValueError("unexpected connector tools response")
        return tools

    def call_tool(
        self,
        connector_id_or_name: str,
        tool_name: str,
        arguments: dict[str, Any],
    ) -> Any:
        if not connector_id_or_name or not tool_name:
            raise ValueError("connector and tool names are required")
        if not isinstance(arguments, dict):
            raise ValueError("tool arguments must be an object")
        connector = quote(connector_id_or_name, safe="")
        tool = quote(tool_name, safe="")
        return self._request(
            "POST",
            f"/v1/connectors/{connector}/tools/{tool}/call",
            {"arguments": arguments},
        )


def apply_plan(
    client: MistralConnectorClient,
    plan: dict[str, Any],
    *,
    allow_org_shared: bool = False,
) -> dict[str, Any]:
    """Apply one safe mutation and prove the resulting state reconciles to NOOP."""
    action = plan.get("action")
    if action not in ACTIONS:
        raise ValueError(f"unknown reconciliation action: {action}")
    if action == "NOOP":
        return {"action": "NOOP", "changed": False}
    if action in {"BLOCKED", "EXTERNAL"}:
        raise ValueError(f"refusing non-mutable reconciliation action: {action}")

    desired = validate_manifest(plan["desired"])
    if desired["visibility"] == "shared_org" and not allow_org_shared:
        raise ValueError("shared_org mutation requires explicit elevation")

    if action == "CREATE":
        mutation = client.create_connector(desired)
        identifier = mutation.get("id") if isinstance(mutation, dict) else None
    elif action == "UPDATE":
        mutation = client.update_connector(plan["connector_id"], plan["changes"])
        identifier = plan["connector_id"]
    else:
        raise ValueError(f"mutation not implemented for action: {action}")

    identifier = str(identifier or desired["name"])
    refreshed = client.get_connector(identifier)
    verification = plan_reconciliation(
        desired,
        refreshed,
        allow_org_shared=allow_org_shared,
    )
    if verification["action"] != "NOOP":
        raise RuntimeError(
            "post-apply verification failed: "
            + json.dumps(sanitize(verification), sort_keys=True)
        )
    return {
        "action": action,
        "changed": True,
        "remote": sanitize(refreshed),
        "verification": verification,
    }


def _print(value: Any) -> None:
    print(json.dumps(sanitize(value), indent=2, sort_keys=True))


def _client_from_env() -> MistralConnectorClient:
    key = os.environ.get("MISTRAL_API_KEY", "")
    if not key:
        raise SystemExit("MISTRAL_API_KEY is required for live operations")
    return MistralConnectorClient(key)


def _find_remote(
    client: MistralConnectorClient, name: str
) -> dict[str, Any] | None:
    return next(
        (item for item in client.list_connectors() if item.get("name") == name),
        None,
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    validate = sub.add_parser(
        "validate", help="validate a local connector manifest or registry"
    )
    validate.add_argument("manifest")

    plan = sub.add_parser("plan", help="plan against supplied remote-state JSON")
    plan.add_argument("manifest")
    plan.add_argument("--remote-file")
    plan.add_argument("--allow-org-shared", action="store_true")

    live = sub.add_parser("live-plan", help="calculate a non-mutating live plan")
    live.add_argument("manifest")
    live.add_argument("--allow-org-shared", action="store_true")

    apply_cmd = sub.add_parser("apply", help="apply and verify a CREATE/UPDATE plan")
    apply_cmd.add_argument("manifest")
    apply_cmd.add_argument("--apply", action="store_true", required=True)
    apply_cmd.add_argument("--allow-org-shared", action="store_true")

    tools = sub.add_parser("tools", help="list tools for one live connector")
    tools.add_argument("connector")
    tools.add_argument("--refresh", action="store_true")

    drift = sub.add_parser("diff-tools", help="compare sanitized tool snapshots")
    drift.add_argument("previous")
    drift.add_argument("current")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.command == "validate":
        _print(load_validation_target(args.manifest))
        return 0
    if args.command == "plan":
        desired = load_manifest(args.manifest)
        remote = _read_json(args.remote_file) if args.remote_file else None
        _print(
            plan_reconciliation(
                desired,
                remote,
                allow_org_shared=args.allow_org_shared,
            )
        )
        return 0
    if args.command == "diff-tools":
        _print(diff_tools(_read_json(args.previous), _read_json(args.current)))
        return 0

    client = _client_from_env()
    if args.command == "tools":
        _print(client.list_tools(args.connector, refresh=args.refresh, pretty=True))
        return 0

    desired = load_manifest(args.manifest)
    remote = _find_remote(client, desired["name"])
    plan = plan_reconciliation(
        desired,
        remote,
        allow_org_shared=args.allow_org_shared,
    )
    if args.command == "live-plan":
        _print(plan)
        return 0
    if args.command == "apply":
        _print(
            apply_plan(
                client,
                plan,
                allow_org_shared=args.allow_org_shared,
            )
        )
        return 0
    raise SystemExit(f"unsupported command: {args.command}")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ManifestError as exc:
        print(f"manifest error: {exc}", file=sys.stderr)
        raise SystemExit(2)
