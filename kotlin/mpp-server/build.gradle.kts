// mpp-server: Ktor JVM server with Server-Sent Events streaming endpoints.
//
// In Ktor 2.x the SSE route DSL (`sse { }`) ships in ktor-server-core; there is
// no separate ktor-server-sse artifact (that artifact was introduced in 3.0).
plugins {
  kotlin("jvm")
  id("org.jlleitschuh.gradle.ktlint")
  application
}

repositories {
  mavenCentral()
}

dependencies {
  implementation(kotlin("stdlib"))
  implementation(platform("io.ktor:ktor-bom:2.3.12"))
  implementation("io.ktor:ktor-server-core")
  implementation("io.ktor:ktor-server-netty")
  implementation("org.slf4j:slf4j-simple:2.0.16")

  testImplementation(kotlin("test"))
  testImplementation(platform("io.ktor:ktor-bom:2.3.12"))
  testImplementation("io.ktor:ktor-server-test-host")
}

application {
  mainClass.set("dev.autodev.server.MainKt")
}

kotlin {
  jvmToolchain(17)
}
