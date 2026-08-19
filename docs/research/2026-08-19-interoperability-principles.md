# Harness interoperability principles

1. Prefer established portable semantics over AutoDev-specific duplication.
2. Preserve source, version, integrity, and transformation provenance on import.
3. Translate external permissions to capability requests; never translate them directly to authorization grants.
4. Fail closed when a source format cannot represent a security-critical AutoDev requirement.
5. Keep internal representation richer than the interchange format when evidence/recovery/trust requires it.
6. Export only semantics that can be represented without misleading loss.
7. Test native and imported equivalent configurations for behavioral parity below the policy boundary.
