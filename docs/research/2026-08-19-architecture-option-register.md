# Architecture option register

## External harness integration

- Embed one external framework as core runtime — rejected direction: couples trusted architecture to framework churn/authority assumptions.
- Implement all formats independently — possible but creates ecosystem fragmentation.
- Federated adapters over normalized internal assets — selected direction; evaluate Harness Protocol as preferred interchange where lossless.

## Durable state

- Transcript as state — rejected.
- PLANS.md prose as machine state — rejected.
- Typed durable state + living narrative plan — selected.

## Android execution

- Phone hosts every workload — rejected as default.
- Android only remote viewer — too weak for local-first goals.
- Capability-aware local/remote hybrid with Android as primary control surface — selected direction.
