# Federated Harness Kernel open questions

Resolve near the milestone that needs each answer; avoid premature framework lock-in.

- Which Harness Protocol v1 semantics can AutoDev adopt losslessly, and which require namespaced extensions?
- Should internal HarnessAsset persistence use one enum document, normalized records, or an event-sourced representation?
- What evidence is sufficient to reconcile an interrupted filesystem/Git/process/network effect?
- Which orchestration state belongs in Rust ForgeCore versus KMP/server control-plane layers?
- How should Android background execution degrade when the OS suspends or kills the process?
- What plugin integrity/signature mechanisms are practical across Android, Termux, desktop, and server environments?
- Which held-out evaluation tasks best detect harness-profile overfitting?
- What minimum evidence should be required before learned routing becomes a default rather than a recommendation?
