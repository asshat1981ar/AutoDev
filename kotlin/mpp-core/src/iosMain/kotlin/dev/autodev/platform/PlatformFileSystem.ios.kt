package dev.autodev.platform

// iOS actual implementation placeholder
actual class PlatformFileSystem actual constructor() {
    actual fun readText(path: String): String {
        throw FileSystemException("iOS readText not implemented in placeholder")
    }

    actual fun writeText(path: String, content: String) {
        throw FileSystemException("iOS writeText not implemented in placeholder")
    }
}
