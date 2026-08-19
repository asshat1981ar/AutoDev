# Architecture fitness functions

Candidate automated fitness functions for later harness enforcement:

- ExecPlan/harness modules cannot import trusted grant constructors.
- Android/commonMain cannot import platform-forbidden execution APIs.
- Imported harness permissions appear only as requested capabilities before policy.
- Required verifier names are known and executed before completion.
- Durable state schemas round-trip frozen fixtures across supported versions.
- Plugin/harness provenance is present for externally sourced activated assets.
- Interrupted effect fixtures cannot transition directly to running without reconciliation evidence.

Fitness functions should move critical architectural rules from prose into executable drift/tests where practical.
