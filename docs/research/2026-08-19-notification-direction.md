# Notification direction

Notifications should focus on actionable durable state: approval required, ambiguous recovery requiring review, verification failure after bounded repair, run completion, or critical environment degradation. Avoid notifying for every agent step.

A notification deep-links to the relevant run/task/evidence/approval state and remains correct if the user opens it after the run has advanced; stale actions must be revalidated.
