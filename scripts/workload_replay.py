#!/usr/bin/env python3
"""Deterministic read-only workload replay for AutoDev control-plane adapters.

The harness is intentionally dependency-free so the same client workload can be
used against Rust, Go, Kotlin, or another HTTP adapter. Remote targets are
blocked by default to avoid accidentally sending MCP bearer tokens off-host.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import dataclasses
import ipaddress
import json
import math
import os
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Iterable


@dataclasses.dataclass(frozen=True)
class ReplayRequest:
    name: str
    method: str
    path: str
    body: bytes = b""


@dataclasses.dataclass(frozen=True)
class RequestSample:
    name: str
    latency_ms: float
    status: int | None
    error: str | None


def build_workload(iterations: int) -> list[ReplayRequest]:
    """Build a deterministic read-only request sequence."""
    requests: list[ReplayRequest] = []
    for index in range(max(0, iterations)):
        requests.append(ReplayRequest("health", "GET", "/health"))
        requests.append(ReplayRequest("objectives_list", "GET", "/api/v1/objectives"))
        mcp_body = json.dumps(
            {
                "jsonrpc": "2.0",
                "id": f"replay-{index}",
                "method": "tools/list",
                "params": {
                    "_meta": {
                        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                        "io.modelcontextprotocol/clientInfo": {
                            "name": "autodev-workload-replay",
                            "version": "1",
                        },
                        "io.modelcontextprotocol/clientCapabilities": {},
                    }
                },
            },
            separators=(",", ":"),
            sort_keys=True,
        ).encode("utf-8")
        requests.append(ReplayRequest("mcp_tools_list", "POST", "/mcp", mcp_body))
    return requests


def percentile(values: Iterable[float], percentile_value: int) -> float:
    """Return the nearest-rank percentile for a finite sample."""
    ordered = sorted(values)
    if not ordered:
        return 0.0
    rank = math.ceil((max(0, min(100, percentile_value)) / 100) * len(ordered))
    index = max(0, rank - 1)
    return float(ordered[index])


def summarize_samples(samples: list[RequestSample], elapsed_seconds: float) -> dict[str, Any]:
    """Summarize latency, throughput, and error metrics."""
    latencies = [sample.latency_ms for sample in samples]
    successes = [
        sample
        for sample in samples
        if sample.error is None and sample.status is not None and 200 <= sample.status < 300
    ]
    errors = len(samples) - len(successes)
    request_count = len(samples)
    elapsed = max(elapsed_seconds, 1e-9)

    by_request: dict[str, dict[str, Any]] = {}
    for name in sorted({sample.name for sample in samples}):
        matching = [sample for sample in samples if sample.name == name]
        matching_latencies = [sample.latency_ms for sample in matching]
        matching_errors = sum(
            1
            for sample in matching
            if sample.error is not None
            or sample.status is None
            or not 200 <= sample.status < 300
        )
        by_request[name] = {
            "requests": len(matching),
            "errors": matching_errors,
            "latency_ms": {
                "p50": round(percentile(matching_latencies, 50), 3),
                "p95": round(percentile(matching_latencies, 95), 3),
                "p99": round(percentile(matching_latencies, 99), 3),
                "max": round(max(matching_latencies, default=0.0), 3),
            },
        }

    return {
        "requests": request_count,
        "successes": len(successes),
        "errors": errors,
        "error_rate_bps": round((errors * 10_000) / request_count) if request_count else 0,
        "throughput_rps": round(request_count / elapsed, 3),
        "latency_ms": {
            "p50": round(percentile(latencies, 50), 3),
            "p95": round(percentile(latencies, 95), 3),
            "p99": round(percentile(latencies, 99), 3),
            "max": round(max(latencies, default=0.0), 3),
        },
        "by_request": by_request,
    }


def parse_proc_stat_cpu_ticks(stat_text: str) -> int:
    """Extract Linux /proc/<pid>/stat user+system CPU ticks."""
    closing = stat_text.rfind(")")
    if closing < 0:
        raise ValueError("invalid /proc stat: missing command terminator")
    fields = stat_text[closing + 1 :].strip().split()
    if len(fields) <= 12:
        raise ValueError("invalid /proc stat: missing CPU fields")
    # fields[0] is kernel field 3 (state), so utime/stime (14/15) are 11/12.
    return int(fields[11]) + int(fields[12])


def parse_vm_rss_kib(status_text: str) -> int:
    """Extract VmRSS from Linux /proc/<pid>/status."""
    for line in status_text.splitlines():
        if line.startswith("VmRSS:"):
            parts = line.split()
            if len(parts) >= 2:
                return int(parts[1])
    raise ValueError("VmRSS not present in /proc status")


def validate_target(target: str, allow_non_loopback: bool) -> str:
    """Validate and normalize a benchmark target origin."""
    parsed = urllib.parse.urlsplit(target)
    if parsed.scheme not in {"http", "https"} or not parsed.hostname:
        raise ValueError("target must be an http(s) origin")
    if parsed.username or parsed.password:
        raise ValueError("target must not embed credentials")
    if parsed.path not in {"", "/"} or parsed.query or parsed.fragment:
        raise ValueError("target must be an origin without path, query, or fragment")

    if not allow_non_loopback and not _is_loopback_host(parsed.hostname):
        raise ValueError("non-loopback target requires --allow-non-loopback")
    return target.rstrip("/")


def _is_loopback_host(host: str) -> bool:
    if host.lower() == "localhost":
        return True
    try:
        return ipaddress.ip_address(host).is_loopback
    except ValueError:
        return False


def _execute_request(
    target: str,
    request: ReplayRequest,
    token: str | None,
    timeout_seconds: float,
) -> RequestSample:
    headers = {"Accept": "application/json"}
    if request.name == "mcp_tools_list":
        headers.update(
            {
                "Accept": "application/json, text/event-stream",
                "Content-Type": "application/json",
                "MCP-Protocol-Version": "2026-07-28",
                "Mcp-Method": "tools/list",
            }
        )
        if token:
            headers["Authorization"] = f"Bearer {token}"

    http_request = urllib.request.Request(
        f"{target}{request.path}",
        data=request.body if request.method == "POST" else None,
        headers=headers,
        method=request.method,
    )
    started = time.perf_counter_ns()
    try:
        with urllib.request.urlopen(http_request, timeout=timeout_seconds) as response:
            response.read(1024 * 1024)
            status = response.status
            error = None if 200 <= status < 300 else f"HTTP {status}"
    except urllib.error.HTTPError as exc:
        status = exc.code
        error = f"HTTP {exc.code}"
    except (urllib.error.URLError, TimeoutError, OSError) as exc:
        status = None
        error = f"{type(exc).__name__}: {exc}"
    elapsed_ms = (time.perf_counter_ns() - started) / 1_000_000
    return RequestSample(request.name, elapsed_ms, status, error)


def _read_cpu_ticks(pid: int) -> int | None:
    try:
        return parse_proc_stat_cpu_ticks(Path(f"/proc/{pid}/stat").read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None


def _read_rss_kib(pid: int) -> int | None:
    try:
        return parse_vm_rss_kib(Path(f"/proc/{pid}/status").read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None


def _run_requests(
    target: str,
    requests: list[ReplayRequest],
    token: str | None,
    concurrency: int,
    timeout_seconds: float,
) -> list[RequestSample]:
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, concurrency)) as executor:
        return list(
            executor.map(
                lambda request: _execute_request(target, request, token, timeout_seconds),
                requests,
            )
        )


def run_replay(
    *,
    target: str,
    token: str | None,
    iterations: int,
    concurrency: int,
    timeout_seconds: float,
    pid: int | None,
    warmup_iterations: int,
) -> dict[str, Any]:
    """Execute the workload and return a machine-readable evidence record."""
    if warmup_iterations > 0:
        _run_requests(
            target,
            build_workload(warmup_iterations),
            token,
            concurrency,
            timeout_seconds,
        )

    start_cpu_ticks = _read_cpu_ticks(pid) if pid else None
    initial_rss = _read_rss_kib(pid) if pid else None
    max_rss = [initial_rss or 0]
    stop_monitor = threading.Event()

    def monitor_rss() -> None:
        if not pid:
            return
        while not stop_monitor.wait(0.01):
            current = _read_rss_kib(pid)
            if current is not None:
                max_rss[0] = max(max_rss[0], current)

    monitor = threading.Thread(target=monitor_rss, name="rss-monitor", daemon=True)
    monitor.start()
    started = time.perf_counter()
    try:
        samples = _run_requests(
            target,
            build_workload(iterations),
            token,
            concurrency,
            timeout_seconds,
        )
    finally:
        elapsed_seconds = time.perf_counter() - started
        stop_monitor.set()
        monitor.join(timeout=1.0)

    end_cpu_ticks = _read_cpu_ticks(pid) if pid else None
    final_rss = _read_rss_kib(pid) if pid else None
    if final_rss is not None:
        max_rss[0] = max(max_rss[0], final_rss)

    process_metrics: dict[str, Any] | None = None
    if pid:
        clock_ticks = os.sysconf("SC_CLK_TCK")
        cpu_seconds = None
        if start_cpu_ticks is not None and end_cpu_ticks is not None:
            cpu_seconds = max(0.0, (end_cpu_ticks - start_cpu_ticks) / clock_ticks)
        process_metrics = {
            "pid": pid,
            "cpu_seconds": round(cpu_seconds, 6) if cpu_seconds is not None else None,
            "cpu_utilization_pct": (
                round((cpu_seconds / max(elapsed_seconds, 1e-9)) * 100, 3)
                if cpu_seconds is not None
                else None
            ),
            "rss_kib_initial": initial_rss,
            "rss_kib_final": final_rss,
            "rss_kib_max": max_rss[0] or None,
        }

    return {
        "schema_version": 1,
        "target": target,
        "workload": {
            "iterations": iterations,
            "requests_per_iteration": 3,
            "concurrency": max(1, concurrency),
            "warmup_iterations": max(0, warmup_iterations),
            "timeout_seconds": timeout_seconds,
            "mutation_free": True,
        },
        "elapsed_seconds": round(elapsed_seconds, 6),
        "summary": summarize_samples(samples, elapsed_seconds),
        "process": process_metrics,
    }


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", default="http://127.0.0.1:18080")
    parser.add_argument("--token", default=os.getenv("AUTODEV_MCP_BEARER_TOKEN"))
    parser.add_argument("--iterations", type=int, default=200)
    parser.add_argument("--concurrency", type=int, default=16)
    parser.add_argument("--warmup-iterations", type=int, default=10)
    parser.add_argument("--timeout", type=float, default=5.0)
    parser.add_argument("--pid", type=int)
    parser.add_argument("--allow-non-loopback", action="store_true")
    parser.add_argument("--output", type=Path)
    return parser.parse_args()


def main() -> int:
    args = _parse_args()
    try:
        target = validate_target(args.target, args.allow_non_loopback)
    except ValueError as exc:
        raise SystemExit(str(exc)) from exc
    if args.iterations < 1:
        raise SystemExit("--iterations must be at least 1")
    if args.concurrency < 1:
        raise SystemExit("--concurrency must be at least 1")
    if args.timeout <= 0:
        raise SystemExit("--timeout must be positive")

    result = run_replay(
        target=target,
        token=args.token,
        iterations=args.iterations,
        concurrency=args.concurrency,
        timeout_seconds=args.timeout,
        pid=args.pid,
        warmup_iterations=max(0, args.warmup_iterations),
    )
    rendered = json.dumps(result, indent=2, sort_keys=True)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered + "\n", encoding="utf-8")
    print(rendered)
    return 0 if result["summary"]["errors"] == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
