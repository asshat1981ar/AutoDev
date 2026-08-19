# Plan persistence direction

Persistence should provide atomic durable snapshots or events with schema versioning and migration support. It must not turn serialized plan content into trusted approval material.

Near-term options to evaluate during durable orchestration:

- append-only plan events + materialized current state;
- transactional snapshot plus transition log;
- integration with existing orchestration persistence if it already satisfies recovery/audit needs.

Selection criteria: crash consistency, migration simplicity, Android/server portability, auditability, and deterministic recovery—not database novelty.
