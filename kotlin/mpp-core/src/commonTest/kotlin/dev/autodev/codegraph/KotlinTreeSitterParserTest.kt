package dev.autodev.codegraph

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class KotlinTreeSitterParserTest {
    private val parser = KotlinTreeSitterParser()

    @Test
    fun extracts_top_level_function() {
        val src = "fun hello() { println(\"hi\") }"
        val graph = parser.parse(src)
        val fns = graph.byKind("function")
        assertEquals(1, fns.size)
        assertEquals("hello", fns.first().name)
    }

    @Test
    fun ignores_identifiers_in_strings_and_comments() {
        val src =
            """
            // fun commented() {}
            val s = "fun inside() {}"
            fun real() {}
            """.trimIndent()
        val graph = parser.parse(src)
        val names = graph.findByName("commented")
        assertTrue(names.isEmpty(), "commented should not be a real symbol")
        assertTrue(graph.findByName("inside").isEmpty())
        assertEquals(1, graph.findByName("real").size)
    }

    @Test
    fun extracts_class_and_nested_function() {
        val src =
            """
            class Foo {
                fun bar() {}
            }
            """.trimIndent()
        val graph = parser.parse(src)
        val classes = graph.byKind("class")
        assertEquals(1, classes.size)
        assertEquals("Foo", classes.first().name)
        val bar = graph.findByName("bar")
        assertEquals(1, bar.size)
        assertEquals("Foo.bar", bar.first().qualifiedName)
    }

    @Test
    fun symbolAt_resolves_offset_inside_identifier_span() {
        // The class identifier "A" occupies a 1-char span. An offset at the
        // identifier start resolves to the class symbol.
        val src = "class A { fun b() {} }"
        val graph = parser.parse(src)
        val classSymbol = graph.byKind("class").first()
        val resolved = graph.symbolAt(classSymbol.span.start)
        assertNotNull(resolved)
        assertEquals("A", resolved.name)
    }

    @Test
    fun symbolAt_returns_null_for_whitespace_between_symbols() {
        // The space immediately after the "A" identifier is inside no span.
        val src = "class A { fun b() {} }"
        val graph = parser.parse(src)
        val aIndex = src.indexOf("A")
        val spaceOffset = src.indexOf(' ', aIndex + 1)
        assertNull(graph.symbolAt(spaceOffset))
    }

    @Test
    fun empty_source_yields_empty_graph() {
        val graph = parser.parse("")
        assertTrue(graph.isEmpty())
    }

    @Test
    fun properties_are_classified() {
        val src = "val x = 1\nvar y = 2"
        val graph = parser.parse(src)
        val props = graph.byKind("property")
        assertEquals(2, props.size)
    }

    @Test
    fun findByQualifiedName_matches_qualified_names() {
        val src = "class A { fun b() {} fun b() {} }"
        val graph = parser.parse(src)
        val qualified = graph.findByQualifiedName("A.b")
        assertEquals(2, qualified.size)
    }

    @Test
    fun symbolAt_returns_null_outside_any_symbol() {
        val src = "fun a() {}"
        val graph = parser.parse(src)
        assertNull(graph.symbolAt(1000))
    }
}
