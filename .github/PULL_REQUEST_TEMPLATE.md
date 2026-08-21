## Summary

<!-- What changes, and why? -->

## Trust-boundary impact

Select exactly one top-level declaration:

- [ ] No trusted execution or authorization changes
- [ ] Trusted-boundary impact present (check all applicable categories below)
  - [ ] ForgeCore / Workspace / AuthorizationGrant changed
  - [ ] MCP / HTTP / SSE / RPC boundary changed
  - [ ] Protocol identity / provenance / evidence changed
  - [ ] Kotlin Multiplatform boundary changed

The no-impact declaration is valid only when the impact-present declaration and every nested boundary-change category remain unchecked.

## Verification

- [ ] Relevant Rust tests
- [ ] Relevant Kotlin tests
- [ ] Relevant Python tests
- [ ] Relevant Node checks
- [ ] python scripts/check_harness_drift.py
- [ ] Adversarial tests where required

## CodeRabbit

- [ ] Findings reviewed
- [ ] Blocking findings resolved or adjudicated
- [ ] ast-grep findings reviewed
- [ ] Review results are not treated as authorization

## Checklist

- [ ] No unrelated changes
- [ ] No secrets added
- [ ] Required evidence is present and passing
