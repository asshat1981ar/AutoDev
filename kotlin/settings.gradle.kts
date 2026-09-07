// AutoDev Kotlin Multiplatform settings.
rootProject.name = "autodev-kotlin"

pluginManagement {
    repositories {
        gradlePluginPortal()
        mavenCentral()
        google()
    }

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
                "org.jetbrains.kotlin.plugin.compose" -> useVersion(kotlinVersion)
                "com.android.application" -> useVersion(agpVersion)
                "org.jlleitschuh.gradle.ktlint" -> useVersion(ktlintVersion)
            }
        }
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "0.8.0"
}

include(
    "mpp-core",
    "mpp-server",
    "mpp-ui",
    "mpp-codegraph",
    "android-command-center",
    "ide-agent-android",
)
