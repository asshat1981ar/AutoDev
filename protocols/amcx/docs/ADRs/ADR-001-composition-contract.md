# ADR-001: AMCX-1 Composition Contract and Authority Separation

## Status
Accepted

## Context
AMX-1 (Agent Memory Exchange) and ECM (Evidentiary Collaboration Mesh) define overlapping yet distinct concerns in agent memory and collaboration. A composition contract is required to define how they interoperate without semantic collisions.

## Decision
1. AMX is the sole canonical portable-memory contract.
2. ECM owns task collaboration workflows, attempts, leases, and ContextViews, but NEVER mutates AMX causal heads.
3. `ECMMemoryBinding` is noncanonical, digest-linked, and rebuildable.
4. ForgeCore alone owns capability authorization and effect execution.
