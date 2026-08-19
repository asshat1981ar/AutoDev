# Fault-injection direction

Fault injection should target realistic boundaries: process death, disk/write failure, network loss, stale repository state, tool timeout, malformed output, verifier outage, approval delay, plugin integrity change, and partial external success.

Tests assert both recovery behavior and that the system does not widen authority or fabricate evidence under failure. Deterministic injection points are preferable to flaky timing-based chaos for CI; broader chaos can complement them later.
