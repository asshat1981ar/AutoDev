# Experimental configuration tuple

For evaluation and reproducibility, treat an agentic configuration as an explicit tuple rather than an unnamed "agent version":

```text
(model,
 harness_profile,
 agent_profile,
 toolset,
 skills,
 workflow,
 context_policy,
 verifier_recipe,
 policy_version)
```

Each component should have stable/versioned identity. Comparisons can then vary one or a controlled subset of factors, improving causal interpretation of evaluation outcomes.
