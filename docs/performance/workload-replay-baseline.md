# AutoDev workload replay baseline

Status: measured experimental evidence; not a production SLA or cross-language benchmark.

Date: 2026-08-17

## Purpose

This baseline replaces purely simulated assumptions for the **currently implemented Rust control-plane server** with measured end-to-end evidence. It does not assign measured values to hypothetical Go, Kotlin-edge, or Flutter topologies.

The permanent replay client is `scripts/workload_replay.py`. The measurement workflow used to collect the results below was intentionally temporary and was removed after evidence capture so hosted-runner performance work does not become a noisy blocking CI gate.

## Workload contract

Each iteration sends the same mutation-free sequence over loopback HTTP:

1. `GET /health`
2. `GET /api/v1/objectives`
3. MCP `tools/list` over `POST /mcp`

The valid runs used:

- release-mode `autodev-server`;
- 20 warmup iterations;
- 300 measured iterations = 900 requests per concurrency level;
- concurrency levels 1, 8, and 32;
- a Python-stdlib load generator so the workload can target any future adapter implementation without sharing its runtime;
- Linux `/proc/<pid>` sampling for server CPU ticks and RSS;
- 5 second per-request timeout;
- bearer authentication for MCP;
- no application mutations.

The harness blocks non-loopback targets unless `--allow-non-loopback` is explicitly supplied, reducing the risk of accidentally sending MCP bearer credentials to an unintended host.

## Environment

Both evidence runs used GitHub-hosted Ubuntu 24.04 runners with 4 online vCPUs. The captured runner reported an AMD EPYC 9V74 processor under Microsoft virtualization.

Release server binary size: **7,784,520 bytes (7.424 MiB)**.

### Evidence run A

- Workflow run: `32075468516`
- Head: `619b0e35995d55dcc77502142e158676eef065bf`
- Artifact digest: `sha256:4000d0e7f4e142b1c885128453aeb4cef8b1e5bd38fdda41e587bd6af36a741c`

| Concurrency | Requests | Errors | Throughput req/s | p95 latency ms | Server CPU % | Max RSS KiB |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 900 | 0 | 2942.542 | 0.411 | 45.773 | 7436 |
| 8 | 900 | 0 | 3051.010 | 4.310 | 50.850 | 7892 |
| 32 | 900 | 0 | 3338.987 | 15.806 | 51.940 | 8300 |

### Evidence run B

- Workflow run: `32075654885`
- Head: `98e95a640b4efcaca2fedabc7401d76425f2df15`
- Artifact digest: `sha256:2c28f634b5d4ad62ad74e6c21c0c26c64f7dfdc84c747efb1a993dab2277e5fd`

| Concurrency | Requests | Errors | Throughput req/s | p95 latency ms | Server CPU % | Max RSS KiB |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 900 | 0 | 2090.248 | 0.550 | 44.127 | 7252 |
| 8 | 900 | 0 | 2294.262 | 5.564 | 48.434 | 7700 |
| 32 | 900 | 0 | 2473.971 | 20.958 | 52.228 | 8144 |

`Server CPU %` is process CPU seconds divided by wall-clock seconds, so 100% represents one fully occupied CPU-equivalent. It is not total-host utilization.

## Hosted-runner variance

Sequential hosted runs showed substantial variation in client-visible throughput and latency while server-side CPU and RSS remained comparatively stable.

| Concurrency | Throughput spread | p95 spread | CPU spread | RSS spread |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 33.87% | 28.93% | 3.66% | 2.51% |
| 8 | 28.31% | 25.40% | 4.87% | 2.46% |
| 32 | 29.76% | 28.03% | 0.55% | 1.90% |

Spread here is the absolute difference between the two runs divided by their mean. With only two executions it is a diagnostic, not a confidence interval.

This pattern is consistent with the cloud-benchmarking problem described by Koch, Japke, and Bermbach in *Duet instrumentation: An Agentic Approach to Improving Sensitivity in Cloud Service Benchmarking* (`arXiv:2605.18397`): sequential cloud measurements are vulnerable to infrastructure noise. Their benchmark design compares versions concurrently on the same machine with synchronized requests and equal resource limits, then evaluates paired relative measurements.

## Failure isolation

Run B also injected an intentionally invalid MCP bearer token while keeping health and objective-list traffic valid at concurrency 32.

- total requests: 300;
- expected MCP authentication failures: 100/100;
- health errors: 0/100;
- objective-list errors: 0/100;
- successful unaffected requests: 200/200;
- server CPU: 38.271%;
- max RSS: 8144 KiB;
- post-replay `/health` check: passed.

This demonstrates bounded failure isolation for the tested authentication failure mode: rejected MCP requests did not prevent concurrent non-MCP reads or terminate the server.

## Interpretation

### Evidence-supported conclusions

1. The current Rust server completed all **5,400 valid replay requests across the two runs with zero errors**.
2. Under the tested workload, measured server CPU remained below 53% of one CPU-equivalent and max RSS remained below 8.2 MiB.
3. Increasing client concurrency from 1 to 32 increased latency substantially but did not approach server CPU saturation.
4. Throughput and latency varied materially between hosted-runner executions while process CPU/RSS were much more stable.
5. The tested MCP authentication failure remained confined to MCP requests.

### Inference

The current replay does **not** identify Rust server execution as the limiting resource. The combination of low server CPU, stable RSS, throughput plateauing, and high between-run client-visible variance indicates that the Python load generator and/or hosted-runner scheduling/noise is material before Rust server capacity is exhausted.

Therefore this evidence does **not** justify adding a Go gateway or Go worker to the production topology.

## Go experiment gate

The bounded-Go experiment remains deferred until all of the following are true:

1. a representative workload or production trace demonstrates a server-side networking/concurrency bottleneck rather than client-generator or environment saturation;
2. the candidate Go boundary preserves ForgeCore as the only trusted execution authority;
3. Rust and Go candidates can be run simultaneously on the same host with equal isolated resource limits;
4. identical requests are synchronized between candidates;
5. paired relative latency/throughput measurements are evaluated across repeated runs, preferably with bootstrap confidence intervals;
6. the candidate improves the actual bottleneck without unacceptable memory, startup, binary-size, security, deployment, or maintenance cost.

Until that gate is met, **Rust + KMP remains the production default and Go remains an experiment-only candidate**.

## Reproducing the replay

Start a release server with `AUTODEV_MCP_BEARER_TOKEN` configured, then run:

```bash
python scripts/workload_replay.py \
  --target http://127.0.0.1:8080 \
  --iterations 300 \
  --warmup-iterations 20 \
  --concurrency 32 \
  --pid <autodev-server-pid> \
  --output performance-evidence.json
```

For a future cross-runtime comparison, use the same replay contract against both candidates. Do not infer a language winner from separate hosted-runner executions; use the paired/duet gate above.
