# IDE Agent Android

First APK milestone for the IDE-Agent project.

This module is an Android control plane over the Planner → Coder → Builder → Reviewer architecture found in the Google Drive `IDE-Agent/android-ide-agent` source. Desktop-only execution is deliberately hidden behind `ExecutionBackend` so Android does not directly depend on `/bin/sh`, Gradle subprocesses, Git CLI, or ADB.

## Current milestone

- Jetpack Compose task UI
- agent pipeline state visualization
- Plan / Changes / Build / Review result surfaces
- deterministic `PreviewExecutionBackend`
- unit tests for pipeline state transitions
- GitHub Actions debug APK build

## Next backend

Implement `TermuxExecutionBackend` without changing UI state contracts. The backend should communicate with an explicit local service/bridge and return structured execution events rather than granting arbitrary shell execution to the APK.

## Build

```bash
cd kotlin
./gradlew :ide-agent-android:testDebugUnitTest :ide-agent-android:assembleDebug
```

APK output:

`kotlin/ide-agent-android/build/outputs/apk/debug/ide-agent-android-debug.apk`
