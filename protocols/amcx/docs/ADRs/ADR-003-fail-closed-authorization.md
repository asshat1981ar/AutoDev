# ADR-003: Fail-Closed Authorization and Secret Handling Invariants

## Status
Accepted

## Context
Handling ambiguous requests, stale evidence, or raw secrets in memory records poses severe security risks.

## Decision
1. Fail-Closed Default: Any absence of authorization, expired lease, stale evidence, or unknown schema MUST immediately fail closed (deny access).
2. Zero Raw Secrets: Raw bearer tokens, API keys, passwords, or session cookies are strictly prohibited in memory records and ContextViews. Only non-secret, purpose-bound, brokered secret references are permitted.
