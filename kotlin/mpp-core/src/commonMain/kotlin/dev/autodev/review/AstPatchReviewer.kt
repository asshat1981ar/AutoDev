package dev.autodev.review

class AstPatchReviewer {
    fun review(original: String, patched: String): List<String> {
        // Return list of issues/warnings
        val issues = mutableListOf<String>()
        if (original == patched) issues.add("No changes detected")
        return issues
    }
}
