# Declarative harness profile principles

- Profiles tune prompts, tool visibility/descriptions, subagent defaults, skill sources, middleware/hooks, and output contracts.
- Profiles are selected by explicit compatibility/routing rules and recorded in run provenance.
- Profile assembly must fail loudly on unknown/excluded load-bearing configuration rather than silently degrade.
- A subagent may narrow requested tools/permissions relative to its parent; any expansion still passes ForgeCore policy.
- Profile changes are evaluated as configuration changes against frozen tasks before becoming defaults.
- Kernel policy, authorization, evidence requirements, and trusted verification are outside the profile-removable surface.
