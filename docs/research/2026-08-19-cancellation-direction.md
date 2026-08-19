# Cancellation direction

Cancellation is a durable intent to stop future work, not proof that every in-flight effect instantly ceased. The orchestrator should stop dispatching new work, attempt cancellation where supported, and reconcile any effect whose terminal state is uncertain.

UI should distinguish `cancelling`/uncertain work from a fully `cancelled` safe checkpoint when later implementation adds richer lifecycle states.
