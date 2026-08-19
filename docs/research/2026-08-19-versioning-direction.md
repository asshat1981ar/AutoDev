# Schema/versioning direction

Durable plan, harness asset/profile, and evidence records require explicit schema versions before they become persisted production contracts. Readers should reject unsupported future versions rather than partially interpreting security-critical fields.

Migrations should be deterministic and tested against frozen fixtures. External harness importers retain source-format version separately from AutoDev internal schema version so round-trip/provenance remain explainable.
