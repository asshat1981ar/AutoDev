# User-facing diagnostics direction

Every non-success terminal/intervention state should answer:

- What happened?
- What work is already safely complete?
- What evidence is missing or failed?
- Is any external effect uncertain?
- What capability/approval is required, if any?
- What will AutoDev do if resumed/retried?
- Has the retry/replan budget been consumed?

Diagnostics should derive from typed state/evidence rather than model-generated explanations alone. Model-generated summaries may improve readability but must link back to the underlying records.
