# State taxonomy

Keep distinct:

- objective/plan state;
- task/dependency state;
- execution-envelope/effect state;
- policy/approval state;
- verification/evidence state;
- environment/harness configuration state;
- client presentation/cache state.

Many agent-system recovery bugs come from collapsing these layers—for example treating a disconnected UI as a failed effect or treating a plan's `running` state as proof an execution process still exists.
