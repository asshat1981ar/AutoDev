package dev.autodev.ui

import kotlin.test.Test
import kotlin.test.assertTrue

class NanoDSLTest {
  @Test
  fun escapes_html_characters() {
    val out = NanoDSL().render("val x = a < b && c > d")
    assertTrue(out.contains("&lt;"))
    assertTrue(out.contains("&gt;"))
    assertTrue(out.contains("&amp;"))
    assertTrue(!out.contains("<b>"))
  }

  @Test
  fun classifies_added_and_removed_lines() {
    val diff = "--- a\n+++ b\n@@\n-removed\n+added\n ctx"
    val out = NanoDSL().render(diff)
    assertTrue(out.contains("autodev-diff-add"))
    assertTrue(out.contains("autodev-diff-del"))
    assertTrue(out.contains("autodev-diff-hunk"))
  }

  @Test
  fun diff_preview_produces_pre_wrapped_output() {
    val out = DiffPreviewNanoDSL().preview("a\nb", "a\nc")
    assertTrue(out.startsWith("<pre class=\"autodev-diff\">"))
    assertTrue(out.contains("autodev-diff-del"))
    assertTrue(out.contains("autodev-diff-add"))
  }
}
