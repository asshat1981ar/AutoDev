# Implementation order rationale

Durable coordination precedes plugin/harness breadth because long-running work needs recovery/evidence semantics before adding more dynamic components. Harness normalization precedes broad adapters to avoid one-off integrations. Evaluation precedes adaptive self-configuration so learning has trustworthy outcomes. Android UX follows core contracts closely enough to shape them, but not so early that Compose state becomes the de facto lifecycle model.

This order optimizes architectural leverage rather than feature count.
