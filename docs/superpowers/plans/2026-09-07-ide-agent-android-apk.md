# IDE-Agent Android APK Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce the first installable IDE-Agent Android debug APK.

**Architecture:** A Compose app models Planner → Coder → Builder → Reviewer state and delegates execution through an `ExecutionBackend` boundary. Preview mode is deterministic; Termux integration follows as a separate backend.

**Tech Stack:** Kotlin 2.0.21, AGP 8.8.2, Gradle 8.10.2, Android SDK 35, Jetpack Compose.

**Spec:** `docs/superpowers/specs/2026-09-07-ide-agent-android-apk-design.md`

## Global Constraints
- minSdk 26; targetSdk/compileSdk 35.
- Java/JVM toolchain 17.
- No direct shell/Gradle/Git/ADB execution from Android UI.
- First milestone is a debug APK with unit tests passing.

### Task 1: Android shell and pipeline model
- [x] Add `ide-agent-android` to Gradle settings.
- [x] Create Android application module and manifest.
- [x] Write state transition tests.
- [x] Implement pipeline state model.

### Task 2: Execution boundary and Compose control plane
- [x] Define `ExecutionBackend` and deterministic preview implementation.
- [x] Build task entry, pipeline visualization, and result tabs.

### Task 3: CI verification
- [x] Add GitHub Actions workflow for unit tests and APK assembly.
- [x] Confirm workflow succeeds and capture artifact.

## Verification record
- Workflow run: `34161146771`
- Gradle: `BUILD SUCCESSFUL`; 41 actionable tasks (40 executed, 1 from cache)
- Artifact ID: `10032686973`
- APK SHA-256: `11efe7e5c6d957912dd6738492eced45dc9c60622fe41a47edbdb08a80896e97`
