# Agentic development-loop contract

A future production loop should preserve this invariant sequence:

```text
objective
 -> durable plan
 -> bounded decomposition
 -> context selection
 -> agent/profile/toolset selection
 -> policy/capability evaluation
 -> effect execution
 -> independent verification
 -> checkpoint
 -> complete | bounded repair/replan | blocked | failed
```

Selection and learning occur before policy, never instead of policy. Verification occurs after effects and is independent of the worker's success claim. Every cycle leaves durable evidence sufficient for recovery and later evaluation.
