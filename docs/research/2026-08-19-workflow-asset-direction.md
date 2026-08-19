# Workflow asset direction

Workflow assets should describe orchestration recipes—phase ordering, dependencies, required roles/evaluators, retry/replan policy hints, and expected outputs—without embedding trusted execution grants.

A workflow is instantiated into durable plan/task state and then governed by normal policy/evidence rules. Version/provenance must be recorded so a resumed run does not silently change workflow semantics after an asset update.
