// AutoDev Kotlin Multiplatform settings.
rootProject.name = "autodev-kotlin"

pluginManagement {
    repositories {
        gradlePluginPortal()
        mavenCentral()
        google()
    }
}

// Enable auto-provisioning of JVM toolchains so `jvmToolchain(17)` works on
// machines that only have a different JDK installed (e.g. CI matrix).
plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "0.8.0"
}

include("mpp-core", "mpp-server", "mpp-ui", "mpp-codegraph", "android-command-center")
