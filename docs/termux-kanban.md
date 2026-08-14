# Cline Kanban on Termux / Android ARM64

Cline Kanban currently depends on upstream `node-pty`, whose published package may not contain an Android ARM64 native binding. On Termux this can fail with an error such as:

```text
Failed to load native module: pty.node
Cannot find module './prebuilds/android-arm64/pty.node'
```

AutoDev includes `scripts/termux-kanban.mjs`, a runtime compatibility launcher that leaves non-Termux platforms unchanged.

## What it does

1. Detects Android/Termux ARM64.
2. Locates the globally installed `kanban` package with `npm root -g`.
3. Probes Kanban's current `node-pty` installation first.
4. If the PTY already loads, makes no changes.
5. If it fails, replaces only `kanban/node_modules/node-pty` with the pinned Android ARM64 compatibility package `@mmmbuto/node-pty-android-arm64@1.1.2` using an npm alias.
6. Installs with lifecycle scripts disabled.
7. Verifies the installed `prebuilds/android-arm64/pty.node` SHA-256.
8. Re-probes `node-pty` before launching Kanban.
9. Runs `cline kanban`, falling back to the `kanban` executable if Cline is not on PATH.

Because the probe runs on every launcher invocation, a future global Kanban update that restores the incompatible upstream `node-pty` is repaired on the next launch.

## First repair

From the AutoDev repository:

```bash
cd ~/AutoDev
git pull
node scripts/termux-kanban.mjs --repair-only
```

Then launch through the compatibility adapter:

```bash
node scripts/termux-kanban.mjs
```

Kanban arguments can be passed after `--`:

```bash
node scripts/termux-kanban.mjs -- --help
```

## Diagnostics

Probe without changing anything:

```bash
node scripts/termux-kanban.mjs --check
```

Force reinstall of the pinned Android PTY:

```bash
node scripts/termux-kanban.mjs --force-repair --repair-only
```

## Optional shell alias

For Bash:

```bash
printf "\nalias cline-kanban='node ~/AutoDev/scripts/termux-kanban.mjs'\n" >> ~/.bashrc
source ~/.bashrc
cline-kanban
```

This alias is intentionally separate from the `cline` executable so the compatibility layer cannot interfere with other Cline commands.

## Security and scope

The Android PTY package is a third-party Termux compatibility fork, not an official Cline or Microsoft package. AutoDev therefore pins the exact package version and verifies the native binary before use. The repair is restricted to Android/Termux ARM64 and to Kanban's own `node_modules/node-pty` directory.

The pinned native binary SHA-256 is:

```text
660a3025230f6035b7b8c000e8cca6ca3992bedaa05f7b165e7c3a5f1ae8ec8a
```
