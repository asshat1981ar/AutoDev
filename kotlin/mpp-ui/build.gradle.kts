// mpp-ui: multiplatform diff/preview rendering via a small Nano DSL.
// Pure commonMain, no JVM APIs — renders to an escaped string view.
plugins {
  kotlin("multiplatform")
  id("org.jlleitschuh.gradle.ktlint")
}

kotlin {
  jvmToolchain(rootProject.extra["jvm.target"].toString().toInt())
  jvm()

  sourceSets {
    val commonMain by getting
    val commonTest by getting {
      dependencies {
        implementation(kotlin("test"))
      }
    }
  }
}
