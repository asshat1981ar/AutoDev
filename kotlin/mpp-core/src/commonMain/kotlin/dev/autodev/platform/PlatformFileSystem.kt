package dev.autodev.platform

/**
 * A minimal, pure-common file read/write contract.
 *
 * Implementations are provided per platform via `actual` declarations.
 * The common API never exposes JVM or Darwin types.
 */
expect class PlatformFileSystem() {
  /**
   * Read the entire file at [path] as UTF-8 text.
   *
   * @throws FileSystemException if the path does not exist or cannot be read.
   */
  fun readText(path: String): String

  /**
   * Write [content] to [path], creating or truncating the file. Parent
   * directories are created if missing.
   *
   * @throws FileSystemException if the file cannot be written.
   */
  fun writeText(
    path: String,
    content: String,
  )

  /**
   * Whether a regular file exists at [path].
   */
  fun exists(path: String): Boolean
}
