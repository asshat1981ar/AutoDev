# ADR-002: Git-Backed Neutral Contract Registry

## Status
Accepted

## Context
Runtime agents must not be permitted to dynamically activate or mutate schemas on the fly.

## Decision
1. Schema publication and activation are exclusively governed by the reviewed Neutral Contract Registry in Git.
2. All schemas (AMX, ECM, AMCX) are defined with strict versioning (`v1`), deterministic JSON schema validation, and detached cryptographic checksums (SHA-256).
3. Agents may propose candidates, but activation requires explicit repository review.
