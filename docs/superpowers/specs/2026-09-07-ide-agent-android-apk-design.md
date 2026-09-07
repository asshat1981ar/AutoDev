# IDE-Agent Android APK Design

## Goal
Create a buildable Android APK that acts as the mobile control plane for the existing Koog IDE-Agent architecture without embedding desktop-only execution assumptions into the Android process.

## Architecture
The Android app owns task entry, pipeline visualization, results, and backend selection. `ExecutionBackend` is the authority boundary between UI/orchestration state and execution. The first implementation uses a deterministic preview backend; Termux and remote runners are replaceable later implementations.

## Invariants
- Android UI never calls `/bin/sh`, Gradle, Git CLI, or ADB directly.
- A backend receives a bounded task and emits stage events plus structured results.
- Planner, Coder, Builder, and Reviewer remain explicit stages.
- First milestone must compile into a debug APK in CI.
- Existing Google Drive IDE-Agent source remains unchanged during this first integration branch.

## Verification
Run JVM unit tests for state transitions and assemble the debug APK in GitHub Actions.
