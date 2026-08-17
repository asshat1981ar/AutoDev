plugins {
    kotlin("multiplatform") version "1.9.10" apply false
    id("org.jetbrains.kotlin.jvm") version "1.9.10" apply false
}

allprojects {
    repositories {
        mavenCentral()
        google()
        jcenter()
    }
}
