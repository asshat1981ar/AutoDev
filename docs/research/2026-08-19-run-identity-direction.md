# Run identity direction

Separate identities for plan, run, task, attempt, envelope, effect, checkpoint, and evidence avoid ambiguity during retries/recovery. A new attempt does not overwrite the identity/history of the failed attempt; a replan does not create a new objective unless semantics actually change.

Stable identity relationships are essential for cross-device Android resume and for evaluation traces.
