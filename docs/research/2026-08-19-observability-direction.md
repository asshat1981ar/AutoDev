# Observability direction

Every durable run should make these relationships inspectable without exposing secrets:

`objective -> plan -> milestone -> task -> agent/profile/toolset -> envelope -> effect -> evidence -> verdict -> checkpoint`.

Events should carry stable identities and timestamps so Android and companion clients can reconstruct timelines without becoming the canonical lifecycle store. User-facing observability should distinguish agent claims from trusted execution records and independent verification results.
