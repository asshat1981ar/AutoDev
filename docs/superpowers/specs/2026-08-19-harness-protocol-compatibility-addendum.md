# Federated Harness Kernel — interoperability addendum

## Status

Accepted addendum to `2026-08-18-federated-harness-kernel-design.md` based on research performed 2026-08-19.

## New constraint

Before AutoDev freezes the external representation of `HarnessAsset`, Milestone 2 must perform a schema-gap analysis against Harness Protocol v1. Where Harness Protocol can losslessly represent an AutoDev asset, AutoDev should support import/export rather than create an incompatible portable format.

ForgeCore-specific trust semantics remain non-negotiable extensions: imported permissions are requests to policy, never authorization; integrity/provenance must be retained; unsupported security semantics fail closed; evidence, recovery, and `AuthorizationGrant` remain kernel-owned.

## Harness profiles

Milestone 3 must also evaluate a declarative profile layer inspired by current Deep Agents harness profiles: provider/model-specific prompt, tool, middleware, subagent, skill, and permission configuration should be data-driven rather than scattered runtime conditionals.

Profiles may tune the untrusted harness surface. They cannot remove ForgeCore security/verification requirements or create authority.
