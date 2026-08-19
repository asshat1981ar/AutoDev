# Android event consumption direction

Android should consume resumable typed event streams plus snapshot endpoints. After disconnect/process death, the client loads a current snapshot and resumes events from a cursor rather than assuming its in-memory timeline is complete.

Compose state derives from these contracts. UI actions submit intent/review decisions through APIs; they do not mutate canonical lifecycle state locally and hope to synchronize later.
