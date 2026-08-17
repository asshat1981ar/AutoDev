package dev.autodev.platform

import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

/**
 * JVM implementation of [PlatformFileSystem] backed by [java.nio.file].
 *
 * All OS primitives are confined to this `jvmMain` actual; commonMain sees
 * only the `expect` contract.
 */
actual class PlatformFileSystem actual constructor() {
    private fun resolve(path: String): Path = Paths.get(path)

    actual fun readText(path: String): String {
        val resolved = resolve(path)
        if (!Files.isRegularFile(resolved)) {
            throw FileSystemException("not a regular file: $path")
        }
        return try {
            Files.readString(resolved, StandardCharsets.UTF_8)
        } catch (e: java.nio.file.NoSuchFileException) {
            throw FileSystemException("file not found: $path")
        } catch (e: Exception) {
            throw FileSystemException("read failed for $path: ${e.message}")
        }
    }

    actual fun writeText(
        path: String,
        content: String,
    ) {
        val resolved = resolve(path)
        try {
            val parent = resolved.parent
            if (parent != null) {
                Files.createDirectories(parent)
            }
            Files.writeString(resolved, content, StandardCharsets.UTF_8)
        } catch (e: Exception) {
            throw FileSystemException("write failed for $path: ${e.message}")
        }
    }

    actual fun exists(path: String): Boolean {
        val resolved = resolve(path)
        return Files.isRegularFile(resolved)
    }
}
