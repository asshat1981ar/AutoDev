# Durable-agent simulation scenarios

Future Simulation/Eval Lab fixtures should include:

1. Process termination immediately before and after a filesystem mutation.
2. Git operation succeeds externally but acknowledgement is lost.
3. Network disconnect while an MCP/tool request is in flight.
4. Human approval remains pending across restart.
5. Plugin/profile changes between checkpoint and resume.
6. Repository HEAD changes externally while a run is offline.
7. Required verifier is unavailable or returns malformed evidence.
8. Context retrieval omits a dependency relevant to a planned patch.
9. Subagent returns success with no independent evidence.
10. Replan budget exhaustion after repeated deterministic failure.

Each scenario should assert both functional recovery and preservation of authority/evidence invariants.
