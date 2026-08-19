# Termux direction

Termux is a first-class constrained environment for developer workflows. Core functionality should avoid mandatory native PTY, Bun, Docker daemon, system Gradle, or desktop GUI assumptions.

When a feature cannot run locally, diagnostics should identify the missing capability and offer a remote-companion or reduced mode rather than failing with an opaque dependency error.
