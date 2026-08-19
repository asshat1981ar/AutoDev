# Durable event direction

Events should describe typed lifecycle facts (`task_started`, `approval_required`, `effect_executed`, `verification_failed`, `checkpoint_created`, `run_interrupted`, etc.) with stable ids and ordering/cursor metadata.

Clients can subscribe/render these events, but authoritative current state is reconstructed/validated by the control plane rather than trusting arbitrary client-emitted events. Event schema versioning is required before public API stabilization.
