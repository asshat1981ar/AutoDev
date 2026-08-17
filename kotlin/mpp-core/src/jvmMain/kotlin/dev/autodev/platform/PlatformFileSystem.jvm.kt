package dev.autodev.platform

import java.nio.file.Files
import java.nio.file.Paths

actual class PlatformFileSystem actual constructor() {
    fun ensurePath(path: String) = Paths.get(path)

    actual fun readText(path: String): String = Files.readString(ensurePath(path))
    actual fun writeText(path: String, content: String) = Files.writeString(ensurePath(path), content)
}
