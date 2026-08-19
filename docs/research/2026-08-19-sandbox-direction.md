# Sandbox direction

Sandboxing should be treated as one enforcement layer beneath declared capabilities, not a replacement for policy. Different platforms may provide different isolation primitives; AutoDev should expose those capabilities/limitations explicitly.

The trusted kernel must fail closed when a requested isolation guarantee is unavailable for a high-risk effect. Android/Termux support therefore needs capability-aware execution strategies rather than pretending desktop container isolation exists everywhere.
