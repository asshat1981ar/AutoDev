# Polyglot Hybrid Simulation

Status: Experimental evidence
Date: 2026-08-17

## Purpose

This model is an assumption-visible, deterministic experiment for deciding whether AutoDev should add Go or Flutter to the production topology. It is not a benchmark of deployed runtimes and must not be cited as production performance evidence.

The simulator uses paired seeds and keeps security violations as a hard disqualification before any weighted utility is considered. A topology that is strictly dominated on success, cost, latency, security, and complexity is removed from consideration before ranking.

## Candidate set

1. `rust_kmp`: Rust authority/server with the existing Kotlin Multiplatform and Android clients.
2. `rust_go_gateway`: Rust authority with a Go stateless MCP/network gateway.
3. `kotlin_edge_rust`: Kotlin edge/control-plane adapter with Rust trusted authority.
4. `rust_bounded_go_worker`: Rust authority with a bounded Go networking/worker specialization.
5. `rust_future_flutter_client`: Rust authority with a future Flutter presentation client.

## Model assumptions

The current base parameters are intentionally simple and inspectable. They are simulator assumptions, not observed measurements.

| Topology | Base success (bps) | Base cost (milliunits) | Base latency (ms) | Complexity |
| --- | ---: | ---: | ---: | ---: |
| Rust + KMP | 8200 | 1000 | 100 | 2 |
| Rust + Go gateway | 8250 | 1250 | 115 | 4 |
| Kotlin edge + Rust | 8000 | 1150 | 110 | 4 |
| Rust + bounded Go worker | 8400 | 1350 | 120 | 4 |
| Rust + future Flutter client | 8100 | 1300 | 125 | 5 |

Thirty paired deterministic seeds add bounded variation. The implementation is authoritative for the exact mixing and aggregation rules.

## Default 30-seed result

| Topology | Success (bps) | Cost (milliunits) | Latency (ms) | Security violations | Complexity |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rust + KMP | 8188 | 1017 | 104 | 0 | 2 |
| Rust + Go gateway | 8245 | 1265 | 120 | 0 | 4 |
| Kotlin edge + Rust | 8009 | 1170 | 115 | 0 | 4 |
| Rust + bounded Go worker | 8405 | 1367 | 124 | 0 | 4 |
| Rust + future Flutter client | 8097 | 1313 | 130 | 0 | 5 |

The Pareto frontier contains `rust_kmp`, `rust_go_gateway`, and `rust_bounded_go_worker`. Under the default secondary weights, `rust_kmp` is selected because its lower cost, latency, and complexity outweigh the simulated success advantage of the Go variants.

The Rust test suite pins the 30-seed selected result so parameter or algorithm drift becomes explicit.

## Sensitivity analysis

The model intentionally tests alternative objective weights rather than treating the default utility as ground truth.

- Default weights select `rust_kmp`.
- An efficiency-heavy policy that doubles cost/latency/complexity penalties also selects `rust_kmp`.
- A success-heavy policy that doubles success weight while halving cost/latency/complexity penalties selects `rust_bounded_go_worker`.

This is an important negative result: the simulator does **not** establish a universal topology winner. It establishes that Rust + KMP is the current default candidate under AutoDev's present simplicity/efficiency bias, while a bounded Go worker remains a credible experiment if production measurements show that incremental success or concurrency capability is worth the extra runtime boundary.

## Decision

**Current default experimental candidate: Rust + KMP.**

This does not reject Go or Flutter permanently. It means they have not yet earned production integration from the current evidence. A Go gateway or worker should enter a real prototype only when repository traces identify a networking/concurrency bottleneck that can be measured against Rust. A Flutter client should enter a real prototype only when target coverage or UI iteration evidence demonstrates value beyond the existing Android/KMP path.

## Required next evidence

Before promoting a Go or Flutter topology:

- replace simulator cost and latency assumptions with measured wall-clock and resource data;
- replay representative AutoDev workloads;
- compare failure isolation and operational complexity;
- test deployment and CI burden;
- run security review of every new process/language boundary;
- re-run the paired evaluation and sensitivity analysis.

Until then, the added runtimes remain experimental and the production authority boundary remains Rust ForgeCore.
