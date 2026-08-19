# Scope discipline for the production program

The production vision is broad, but implementation PRs remain slice-sized. Research may map future architecture ahead of implementation; production code should land only when its prerequisite milestone and verification contract are active.

Do not combine persistence, plugin loaders, Android UI, adaptive learning, and orchestration changes merely because they share the long-term vision. Shared contracts should be introduced at the smallest point where a real consumer/test needs them.
