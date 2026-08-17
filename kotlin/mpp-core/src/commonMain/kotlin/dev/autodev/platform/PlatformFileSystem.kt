package dev.autodev.platform

expect class PlatformFileSystem() {
    fun readText(path: String): String
    fun writeText(path: String, content: String)
}
