#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KOTLIN_DIR="$ROOT_DIR/kotlin"
APK_PATH="$KOTLIN_DIR/android-command-center/build/outputs/apk/debug/android-command-center-debug.apk"

if [[ -z "${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}" ]]; then
  echo "ERROR: ANDROID_HOME or ANDROID_SDK_ROOT must point to an Android SDK." >&2
  exit 1
fi

cd "$KOTLIN_DIR"
./gradlew :android-command-center:assembleDebug --no-daemon

if [[ ! -f "$APK_PATH" ]]; then
  echo "ERROR: expected APK was not produced at $APK_PATH" >&2
  exit 1
fi

echo "APK: $APK_PATH"
