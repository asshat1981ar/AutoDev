# Controller context-loss recovery

On session/context loss, do not infer progress from memory. Read the repository program status/ledger/resume pointer, inspect current branch/PR/head, and fetch CI/review evidence. Continue from the first unmet milestone gate.

This prevents re-dispatching already completed development work and makes the repository—not chat history—the durable coordination source.
