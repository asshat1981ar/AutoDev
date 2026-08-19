# Plugin trust model research note

Proposed trust metadata is orthogonal to capability authorization.

- `built_in`: distributed with the AutoDev release.
- `local`: sourced from an explicitly selected local workspace.
- `verified`: provenance/integrity satisfies configured verification policy.
- `untrusted`: discoverable metadata may be inspected, but activation remains constrained.

No trust label grants unrestricted effects. Each activated asset declares requested capabilities; ForgeCore policy evaluates those requests at execution time. Integrity/provenance evidence should be attached to activation and evaluation records so later failures can identify the exact plugin artifact involved.

Future work should compare this model with Harness Protocol integrity/governance fields before freezing wire semantics.
