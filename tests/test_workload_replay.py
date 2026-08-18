import unittest

from scripts.workload_replay import (
    RequestSample,
    build_workload,
    parse_proc_stat_cpu_ticks,
    parse_vm_rss_kib,
    percentile,
    summarize_samples,
    validate_target,
)


class WorkloadReplayTests(unittest.TestCase):
    def test_build_workload_is_deterministic_and_read_only(self):
        first = build_workload(2)
        second = build_workload(2)

        self.assertEqual(first, second)
        self.assertEqual(len(first), 6)
        self.assertEqual(
            [request.name for request in first],
            [
                "health",
                "objectives_list",
                "mcp_tools_list",
                "health",
                "objectives_list",
                "mcp_tools_list",
            ],
        )
        self.assertTrue(all(request.method == "GET" for request in first[:2]))
        self.assertEqual(first[2].method, "POST")
        self.assertNotIn("tools/call", first[2].body.decode("utf-8"))

    def test_percentile_uses_nearest_rank(self):
        values = [10.0, 20.0, 30.0, 40.0]
        self.assertEqual(percentile(values, 50), 20.0)
        self.assertEqual(percentile(values, 95), 40.0)
        self.assertEqual(percentile(values, 99), 40.0)

    def test_summary_separates_success_and_error_metrics(self):
        samples = [
            RequestSample("health", 10.0, 200, None),
            RequestSample("health", 20.0, 200, None),
            RequestSample("mcp_tools_list", 30.0, 401, "HTTP 401"),
            RequestSample("objectives_list", 40.0, None, "timeout"),
        ]

        summary = summarize_samples(samples, elapsed_seconds=0.5)

        self.assertEqual(summary["requests"], 4)
        self.assertEqual(summary["successes"], 2)
        self.assertEqual(summary["errors"], 2)
        self.assertEqual(summary["error_rate_bps"], 5000)
        self.assertEqual(summary["throughput_rps"], 8.0)
        self.assertEqual(summary["latency_ms"]["p50"], 20.0)
        self.assertEqual(summary["latency_ms"]["p95"], 40.0)
        self.assertEqual(summary["latency_ms"]["p99"], 40.0)

    def test_proc_parsers_extract_cpu_ticks_and_rss(self):
        stat = "1234 (autodev-server) S 1 2 3 4 5 6 7 8 9 10 120 30 0 0 0 0 0"
        status = "Name:\tautodev-server\nVmRSS:\t   24576 kB\n"

        self.assertEqual(parse_proc_stat_cpu_ticks(stat), 150)
        self.assertEqual(parse_vm_rss_kib(status), 24576)

    def test_target_is_loopback_by_default(self):
        self.assertEqual(
            validate_target("http://127.0.0.1:8080", False),
            "http://127.0.0.1:8080",
        )
        self.assertEqual(
            validate_target("http://localhost:8080/", False),
            "http://localhost:8080",
        )

        with self.assertRaises(ValueError):
            validate_target("https://example.com", False)

        self.assertEqual(validate_target("https://example.com", True), "https://example.com")


if __name__ == "__main__":
    unittest.main()
