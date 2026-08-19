# Rust/ForgeCore direction

Keep security-critical lifecycle, capability/policy, workspace confinement, execution envelopes, evidence primitives, and deterministic coordination invariants in small typed Rust modules with adversarial tests.

Avoid turning ForgeCore into a framework-specific agent runtime. External agent/harness ecosystems integrate through adapters above the trusted boundary, preserving the kernel's model-agnostic role.
