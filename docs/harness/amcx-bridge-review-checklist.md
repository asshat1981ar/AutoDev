# AMCX Bridge Review Checklist

Review `crates/forge-core/src/amcx_bridge.rs` and `crates/forge-core/tests/amcx_bridge.rs` against these invariants before integration:

- [ ] No import, construction, return, or mutation of `AuthorizationGrant`.
- [ ] No call to ForgeCore execution entry points.
- [ ] No filesystem, network, process, Git, MCP, or deployment effects.
- [ ] No mutation of `ExecPlan`, `PlanCheckpoint`, `Evidence`, `VerificationReport`, or `ContextPack`.
- [ ] Evidence projection rejects a failed fingerprint check.
- [ ] Source repository, revision, and worktree identities fail closed when blank.
- [ ] Context projection carries immutable artifact reference/digest and metadata only; repository file contents are not copied.
- [ ] Verification PASS remains evidence-only and does not become approval/authorization state.
- [ ] No new dependency is added for the bridge.
- [ ] Full Rust workspace and harness-drift gates pass.
