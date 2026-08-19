# Target production release gates

A future production release should require evidence for:

- reproducible Rust/Kotlin/Python/Node builds where applicable;
- signed/integrity-traceable release inputs and plugin assets;
- Android APK/AAB build plus installation/startup smoke test;
- process-death and interrupted-effect recovery tests;
- offline queue/reconnect reconciliation tests;
- policy/approval adversarial tests;
- representative historical coding-agent evaluation baseline;
- accessibility checks for primary Android workflows;
- user-facing failure diagnostics for blocked, interrupted, denied, and exhausted runs;
- documentation/onboarding verification from a clean environment;
- no regression in the production-evolution milestone scorecard's authority and verification dimensions.
