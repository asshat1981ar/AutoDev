// AutoDev Kotlin Multiplatform root build.
//
// KMP 2.x control plane: mpp-core, mpp-server, mpp-ui, mpp-codegraph.
// commonMain stays pure (no JVM/Android APIs); OS primitives live behind
// expect/actual contracts resolved per target.

plugins {
    kotlin("multiplatform") version "2.0.21" apply false
    kotlin("jvm") version "2.0.21" apply false
    id("org.jlleitschuh.gradle.ktlint") version "12.1.1" apply false
}

allprojects {
    repositories {
        mavenCentral()
        google()
    }
}
