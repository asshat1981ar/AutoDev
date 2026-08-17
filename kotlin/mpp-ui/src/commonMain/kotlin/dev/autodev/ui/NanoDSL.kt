package dev.autodev.ui

class NanoDSL {
    fun render(diff: String): String {
        // Placeholder rendering to a small HTML snippet
        return "<pre>" + diff.replace("<", "&lt;") + "</pre>"
    }
}
