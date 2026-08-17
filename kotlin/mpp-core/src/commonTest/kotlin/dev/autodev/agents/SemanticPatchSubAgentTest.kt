package dev.autodev.agents

import kotlin.test.Test
import kotlin.test.assertTrue

class SemanticPatchSubAgentTest {
    @Test
    fun reports_added_and_removed_symbols() {
        val agent = SemanticPatchSubAgent()
        val original = "class Foo { fun a() {} }"
        val patched = "class Foo { fun a() {} fun b() {} }"
        val analysis = agent.analyze(original, patched)
        assertTrue(analysis.addedSymbols.any { it.contains(".b") })
        assertTrue(analysis.removedSymbols.isEmpty())
    }

    @Test
    fun no_change_yields_warning_finding() {
        val agent = SemanticPatchSubAgent()
        val src = "fun a() {}"
        val analysis = agent.analyze(src, src)
        assertTrue(analysis.reviewFindings.any { it.message.contains("no changes") })
    }
}
