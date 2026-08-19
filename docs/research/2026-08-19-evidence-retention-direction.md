# Evidence retention direction

Retain enough evidence to audit/reproduce important outcomes without indefinitely storing every transient model/tool message. Structured verification results, artifact hashes/refs, policy decisions, configuration provenance, and failure/recovery evidence are higher-value durable records.

Retention policy may differ for local development versus shared/server environments and should be configurable without deleting records still required for an active run or release audit.
