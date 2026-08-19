# Secret handling direction

Plans, harness manifests, logs, and evidence should store secret requirements/references, not secret values. Secret resolution belongs at the trusted execution boundary or an approved credential provider with least-privilege scope.

Imported environment definitions marked sensitive should remain metadata until resolved. Diagnostics must redact secret material while preserving enough provenance to explain which credential/resource was required or denied.
