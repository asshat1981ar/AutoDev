# Hooks/middleware direction

Hooks/middleware are useful for telemetry, prompt/context shaping, tool filtering, rate limits, approvals, and workflow policy, but ordering is semantically significant.

Future HarnessAsset support should therefore represent hook phase/order and provenance explicitly. Load-bearing ForgeCore security/verification behavior must not be implemented as removable untrusted middleware. Unknown hook ordering conflicts should fail validation rather than silently reorder security-sensitive behavior.
