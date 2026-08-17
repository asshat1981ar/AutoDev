package dev.autodev.review

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class AstPatchReviewerTest {
    private val reviewer = AstPatchReviewer()

    @Test
    fun no_change_yields_warning() {
        val findings = reviewer.review("fun a() {}", "fun a() {}")
        assertEquals(1, findings.size)
        assertEquals(ReviewSeverity.Warning, findings.first().severity)
    }

    @Test
    fun whitespace_only_is_info() {
        val findings = reviewer.review("fun a(){}", "fun  a( ) {}")
        assertTrue(
            findings.any {
                it.severity == ReviewSeverity.Info && it.message.contains("whitespace")
            },
        )
    }

    @Test
    fun unbalanced_braces_are_error() {
        val findings = reviewer.review("fun a() {", "fun a() ")
        assertTrue(
            findings.any {
                it.severity == ReviewSeverity.Error && it.message.contains("brace")
            },
        )
    }

    @Test
    fun clean_patch_has_no_errors() {
        val findings = reviewer.review("fun a() {}", "fun a() { /* x */ }")
        assertTrue(findings.none { it.severity == ReviewSeverity.Error })
    }
}
