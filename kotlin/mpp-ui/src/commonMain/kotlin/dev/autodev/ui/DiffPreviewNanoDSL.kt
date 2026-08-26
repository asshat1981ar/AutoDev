package dev.autodev.ui

/**
 * Renders a two-pane diff preview (original vs patched) as a single HTML view.
 *
 * Pure commonMain; depends only on [NanoDSL].
 */
public class DiffPreviewNanoDSL {
  private val dsl: NanoDSL = NanoDSL()

  /**
   * Build a minimal unified diff from [original] and [patched], then render.
   *
   * This is a line-oriented diff (Myers would be heavier); it classifies
   * each line as added, removed, or context relative to a set comparison.
   */
  public fun preview(
    original: String,
    patched: String,
  ): String {
    val diff = buildUnifiedDiff(original, patched)
    return dsl.render(diff)
  }

  private fun buildUnifiedDiff(
    original: String,
    patched: String,
  ): String {
    val a = original.split('\n')
    val b = patched.split('\n')
    val sb = StringBuilder()
    sb.append("--- original\n+++ patched\n")
    val maxIdx = maxOf(a.size, b.size) - 1
    for (i in 0..maxIdx) {
      val left = a.getOrNull(i)
      val right = b.getOrNull(i)
      when {
        left == null -> sb.append('+').append(right).append('\n')
        right == null -> sb.append('-').append(left).append('\n')
        left != right -> {
          sb.append('-').append(left).append('\n')
          sb.append('+').append(right).append('\n')
        }
        else -> sb.append(' ').append(left).append('\n')
      }
    }
    return sb.toString().trimEnd('\n')
  }
}
