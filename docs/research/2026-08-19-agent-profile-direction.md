# AgentProfile direction

An AgentProfile should be a declarative role/configuration asset containing identity, purpose, model requirements/profile selector, skill sources, requested tools/capabilities, context policy, output contract, retry constraints, and provenance.

Agent profiles should be composable with provider/model HarnessProfiles without conflating the two: AgentProfile describes the worker role; HarnessProfile tunes the harness for the resolved model/provider. Both remain below ForgeCore policy.
