// AutoDev Kotlin Multiplatform root build.
//
// KMP 2.x control plane: mpp-core, mpp-server, mpp-ui, mpp-codegraph.
// The Android command center is a thin platform application over the shared
// control-plane contracts; platform-specific UI/network code stays isolated.
//
// Centralized configuration flows through kotlin/gradle.properties
// (mirror-fail-closed enforced by scripts/check_harness_drift.py).
// Every version-pinned value below is read from gradle.properties; the
// fallbacks exist only to make a freshly-cloned project build even if
// the properties file is missing.
//
// Plugin versions are NOT declared inline here. They are resolved by
// settings.gradle.kts's pluginManagement.resolutionStrategy.eachPlugin block
// from kotlin/gradle.properties. This avoids the Gradle Kotlin DSL gotcha
// that the `plugins { }` block in build.gradle.kts cannot see top-level
// script bindings nor use `rootProject` (it is evaluated in an isolated
// compilation context). See settings.gradle.kts and
// docs/architecture/KOTLIN_CONFIG_INTEGRATION.md for the rationale.

plugins {
    kotlin("multiplatform") apply false
    kotlin("jvm") apply false
    kotlin("android") apply false
    id("org.jetbrains.kotlin.plugin.compose") apply false
    id("com.android.application") apply false
    id("org.jlleitschuh.gradle.ktlint") apply false
}

allprojects {
    repositories {
        mavenCentral()
        google()
    }

    // Propagate the centralized JVM target as a project extra so module
    // build scripts can reference it without duplicating the literal.
    val jvmTarget: String = providers.gradleProperty("jvm.target").getOrElse("17")
    extra["jvm.target"] = jvmTarget
}
