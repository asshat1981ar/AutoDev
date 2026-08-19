# Harness interoperability boundary

Future harness adapters normalize external configuration into AutoDev coordination/configuration objects. They do not execute imported authority.

```text
Harness Protocol / Deep Agents / Codex / Cline / other harness
                    |
                    v
             import + validate
                    |
                    v
          AutoDev HarnessAsset/Profile
                    |
             capability request
                    v
              ForgeCore policy
```

Lossless interoperability is preferred when an external standard expresses the required semantics. AutoDev extensions remain necessary for kernel-owned authorization, evidence, provenance, recovery, and fail-closed policy behavior.
