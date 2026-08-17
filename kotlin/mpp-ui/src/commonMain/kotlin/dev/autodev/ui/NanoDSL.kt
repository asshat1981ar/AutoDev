package dev.autodev.ui

/**
 * A tiny, dependency-free rendering DSL for unified diffs.
 *
 * Renders a unified-diff string into escaped HTML where added lines (`+...`),
 * removed lines (`-...`), and hunk headers (`@@ ...`) are highlighted. The
 * output is plain commonMain string, so it works on every KMP target.
 */
public class NanoDSL {
    /**
     * Render [diff] (a unified diff) as an escaped, line-classified HTML
     * fragment wrapped in `<pre>`.
     */
    public fun render(diff: String): String {
        val sb = StringBuilder()
        sb.append("<pre class=\"autodev-diff\">")
        diff.split('\n').forEach { line ->
            val (cls, content) = classify(line)
            sb.append("<span class=\"").append(cls).append("\">")
                .append(escape(content))
                .append("</span>\n")
        }
        sb.append("</pre>")
        return sb.toString()
    }

    private fun classify(line: String): Pair<String, String> =
        when {
            line.startsWith("@@") -> "autodev-diff-hunk" to line
            line.startsWith("+") && !line.startsWith("++") -> "autodev-diff-add" to line
            line.startsWith("-") && !line.startsWith("--") -> "autodev-diff-del" to line
            else -> "autodev-diff-ctx" to line
        }

    private fun escape(text: String): String =
        buildString(text.length + 8) {
            for (c in text) {
                when (c) {
                    '<' -> append("&lt;")
                    '>' -> append("&gt;")
                    '&' -> append("&amp;")
                    '"' -> append("&quot;")
                    else -> append(c)
                }
            }
        }
}
