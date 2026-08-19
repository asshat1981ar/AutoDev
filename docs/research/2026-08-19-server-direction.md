# Server/control-plane direction

The server exposes durable coordination/resources and translates validated client intent to kernel operations. It should not duplicate ForgeCore policy in ad hoc endpoint code. Stateless transport endpoints can sit above durable storage/orchestration state.

Server APIs should remain usable by Android/KMP and automation clients without requiring a browser-specific control plane.
