# Project memory direction

Project memory should favor explicit durable artifacts: ADRs, failure records, architecture facts, verified configuration outcomes, and repository-local instructions. Retrieval should surface source/provenance and freshness.

Free-form remembered conclusions that conflict with current repository state are stale evidence, not truth. The system should prefer current code/tests/specs and update or retire stale memory artifacts when discovered.
