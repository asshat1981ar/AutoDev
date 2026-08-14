# Cline Development Fabric

This project-local package makes Cline native-first: use direct tools for local work,
Skills for expertise, Hooks for deterministic safety, Plugins for local reusable tools,
MCP profiles only for external systems, and Teams/Kanban only when parallelism earns its cost.

## Routing

* **S0** direct implementation
* **S1** Plan Mode
* **S2** Deep Planning followed by Teams
* **S3** Deep Planning followed by Kanban worktrees

The loop is FRAME → RECON → PLAN → IMPLEMENT → VERIFY → REVIEW → INTEGRATE → LEARN.
Checkpoints and Auto Compact remain the persistence mechanism; no duplicate memory system is required.

Hooks are configured through Cline's current project-local customization surface. Plugin behavior
in IDE versions may vary; the manifest remains useful in CLI/SDK/Kanban runtimes.