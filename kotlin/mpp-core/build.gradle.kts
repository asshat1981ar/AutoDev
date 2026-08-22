// mpp-core: shared control-plane primitives — code graph, platform filesystem,
// MCP tool handler, AST patch review. Pure commonMain + jvm/ios actuals.
plugins {
  kotlin("multiplatform")
  id("org.jlleitschuh.gradle.ktlint")
}

kotlin {
  jvmToolchain(rootProject.extra["jvm.target"].toString().toInt())
  jvm()
  iosX64()
  iosArm64()
  iosSimulatorArm64()

  sourceSets {
    val commonMain by getting
    val commonTest by getting {
      dependencies {
        implementation(kotlin("test"))
      }
    }
    val jvmTest by getting {
      dependencies {
        implementation(kotlin("test"))
      }
    }
  }
}
