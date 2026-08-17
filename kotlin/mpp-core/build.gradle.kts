plugins {
    kotlin("multiplatform")
}

kotlin {
    jvm()
    ios()
    sourceSets {
        val commonMain by getting {
            dependencies {
            }
        }
    }
}
