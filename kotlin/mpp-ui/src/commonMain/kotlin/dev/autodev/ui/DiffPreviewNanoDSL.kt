package dev.autodev.ui

class DiffPreviewNanoDSL {
    fun preview(original: String, patched: String): String {
        val diff = "--- original\n+++ patched\n" // placeholder
        return NanoDSL().render(diff)
    }
}
