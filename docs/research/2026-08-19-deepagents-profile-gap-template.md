# Deep Agents profile → AutoDev profile matrix template

| Deep Agents concern | AutoDev profile concern | Classification | Notes |
| --- | --- | --- | --- |
| provider/model key | model routing/profile selector | pending | Must remain declarative |
| prompt prefix/suffix | Prompt assets | pending | Untrusted behavior tuning |
| tool inclusion/exclusion | tool surface request | pending | Cannot bypass policy |
| middleware | Hook/Workflow assets | pending | Need ordering and provenance semantics |
| subagent configuration | AgentProfile | pending | Isolate context and requested tools |
| skills | Skill assets | pending | Preserve source precedence |
| filesystem permissions | capability request | pending | ForgeCore policy is final authority |
| interrupt_on | approval/review policy hint | pending | Cannot itself approve |
| structured response | output contract | pending | Validate before orchestration consumes |
