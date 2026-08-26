package dev.autodev.platform

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class PlatformFileSystemTest {
  @Test
  fun write_and_read_round_trip() {
    val fs = PlatformFileSystem()
    val path = createTempFile()
    fs.writeText(path, "hello autodev")
    assertTrue(fs.exists(path))
    assertEquals("hello autodev", fs.readText(path))
  }

  @Test
  fun missing_file_read_throws() {
    val fs = PlatformFileSystem()
    assertFailsWith<FileSystemException> {
      fs.readText("/nonexistent/path/does/not/exist.txt")
    }
  }

  @Test
  fun exists_false_for_missing() {
    val fs = PlatformFileSystem()
    assertTrue(!fs.exists("/nonexistent/" + System.nanoTime()))
  }

  private fun createTempFile(): String {
    val dir = System.getProperty("java.io.tmpdir")
    return java.io.File(dir, "autodev-fs-test-${System.nanoTime()}.txt").absolutePath
  }
}
