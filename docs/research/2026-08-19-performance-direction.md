# Performance direction

Optimize for time-to-verified-outcome, not raw agent throughput. Measure orchestration overhead, model latency, context growth, verifier time, Android rendering/network overhead, and recovery cost separately.

Caching is acceptable only with explicit invalidation/provenance. Fast stale context or cached policy/evidence that no longer applies is worse than slower correct execution. Performance experiments remain subordinate to authority and correctness gates.
