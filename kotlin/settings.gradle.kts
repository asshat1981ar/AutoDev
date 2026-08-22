// AutoDev Kotlin Multiplatform settings.
rootProject.name = "autodev-kotlin"

pluginManagement {
    repositories {
        gradlePluginPortal()
        mavenCentral()
        google()
    }

    // Resolve every plugin version from kotlin/gradle.properties so the
    // root build.gradle.kts can declare plugins without inline `version`
    // literals. The `plugins { }` block in build.gradle.kts is evaluated in
    // an isolated compilation context and cannot see top-level script
    // bindings nor `rootProject` - this is the Gradle-recommended way to
    // centralize plugin versions. The mirror-fail-closed parity check in
    // scripts/check_harness_drift.py keeps config/kotlin/gradle.properties
    // and kotlin/gradle.properties in lockstep.
    val kotlinVersion: String =
        providers.gradleProperty("kotlin.version").getOrElse("2.0.21")
    val ktlintVersion: String =
        providers.gradleProperty("ktlint.version").getOrElse("12.1.1")
    val agpVersion: String =
        providers.gradleProperty("agp.version").getOrElse("8.8.2")

    resolutionStrategy {
        eachPlugin {
            when (requested.id.id) {
                "org.jetbrains.kotlin.multiplatform",
                "org.jetbrains.kotlin.jvm",
                "org.jetbrains.kotlin.android",
                "org.jetbrains.kotlin.plugin.compose" ->
                    useVersion(kotlinVersion)
                "com.android.application" ->
                    useVersion(agpVersion)
                "org.jlleitschuh.gradle.ktlint" ->
                    useVersion(ktlintVersion)
            }
        }
    }
}

// Enable auto-provisioning of JVM toolchains so `jvmToolchain(17)` works on
// machines that only have a different JDK installed (e.g. CI matrix).
plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "0.8.0"
}

include("mpp-core", "mpp-server", "mpp-ui", "mpp-codegraph", "android-command-center")
