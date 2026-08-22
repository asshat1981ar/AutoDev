// mpp-codegraph: deterministic symbol-graph query engine.
//
// A bounded, dependency-free symbol graph over source text: extracts
// declarations via a structural tokenizer and answers name/scope queries.
// Pure commonMain — no tree-sitter native dependency, so it compiles on every
// target. Designed as the query surface that a future tree-sitter-backed
// extractor can plug into.
plugins {
  kotlin("multiplatform")
  id("org.jlleitschuh.gradle.ktlint")
}

kotlin {
  jvmToolchain(17)
  jvm()

  sourceSets {
    val commonMain by getting {
      dependencies {
        implementation(project(":mpp-core"))
      }
    }
    val commonTest by getting {
      dependencies {
        implementation(kotlin("test"))
      }
    }
  }
}
