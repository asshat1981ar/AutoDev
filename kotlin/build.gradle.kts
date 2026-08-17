// AutoDev Kotlin Multiplatform root build.
//
// KMP 2.x control plane: mpp-core, mpp-server, mpp-ui, mpp-codegraph.
// The Android command center is a thin platform application over the shared
// control-plane contracts; platform-specific UI/network code stays isolated.

plugins {
    kotlin("multiplatform") version "2.0.21" apply false
    kotlin("jvm") version "2.0.21" apply false
    kotlin("android") version "2.0.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.0.21" apply false
    id("com.android.application") version "8.8.2" apply false
    id("org.jlleitschuh.gradle.ktlint") version "12.1.1" apply false
}

allprojects {
    repositories {
        mavenCentral()
        google()
    }
}
