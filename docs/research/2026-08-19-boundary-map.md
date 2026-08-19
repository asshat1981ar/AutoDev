# Trust boundary map

```text
UNTRUSTED / ADVISORY
user objective, model output, agent plans, skills, prompts,
external harness manifests, plugin metadata, MCP/tool descriptions,
evaluation candidate claims

        | typed validation / requested capabilities
        v
TRUSTED CONTROL
ForgeCore policy, workspace confinement, AuthorizationGrant,
execution adapters, lifecycle validation

        | execution records
        v
INDEPENDENT EVIDENCE
verifiers, artifact hashes, tests/build/static/security results,
reconciliation observations

        | evidence-gated transitions
        v
DURABLE ORCHESTRATION / CLIENT VIEW
ExecPlan, TaskGraph, checkpoints, Android/KMP/server state
```

Durable orchestration is trusted to represent validated lifecycle facts but does not itself create effect authorization.
