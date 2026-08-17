package dev.autodev.codegraph

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class SymbolGraphQueryEngineTest {
    private val engine =
        SymbolGraphQueryEngine(
            KotlinTreeSitterParser().parse(
                """
                class Foo {
                    fun bar() {}
                    val baz = 1
                }
                fun top() {}
                """.trimIndent(),
            ),
        )

    @Test
    fun summary_counts_kinds() {
        val s = engine.summary()
        assertEquals(2, s.functions)
        assertEquals(1, s.classes)
        assertEquals(1, s.properties)
        assertEquals(4, s.totalSymbols)
    }

    @Test
    fun members_of_scope() {
        val members = engine.membersOf("Foo")
        assertEquals(2, members.size)
        assertTrue(members.map { it.name }.containsAll(listOf("bar", "baz")))
    }

    @Test
    fun declarations_filtered_by_kind() {
        val fns = engine.declarations("function")
        assertEquals(2, fns.size)
    }

    @Test
    fun resolve_returns_symbol_at_identifier_offset() {
        // Resolve at the identifier span of the single declaration.
        val src = "fun top() {}"
        val graph = KotlinTreeSitterParser().parse(src)
        val engine = SymbolGraphQueryEngine(graph)
        val topNameIndex = src.indexOf("top")
        val sym = engine.resolve(topNameIndex)
        assertNotNull(sym)
        assertEquals("top", sym.name)
    }

    @Test
    fun declares_reports_membership() {
        assertTrue(engine.declares("bar"))
        assertTrue(!engine.declares("absent"))
    }
}
