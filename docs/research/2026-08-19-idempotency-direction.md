# Effect idempotency direction

Where tools/services support idempotency keys or stable operation identifiers, AutoDev should bind them to execution-envelope identity and persist them through recovery. This can make reconciliation safer but does not eliminate the need to inspect effect state after ambiguous failures.

Filesystem/Git operations without native idempotency require effect-specific pre/post evidence such as hashes, refs, status, or transaction/checkpoint semantics.
