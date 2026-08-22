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

  @Test
  fun companion_object_body_does_not_produce_phantom_class_symbol() {
    // Regression: previously, parsing `companion object { val x = 1 }`
    // emitted a class symbol named `x` (the parser scanned past the
    // opening brace and bound the first body identifier to `object`).
    val src =
      """
      class Outer {
          companion object {
              val x = 1
          }
      }
      """.trimIndent()
    val graph = parser.parse(src)
    val phantom = graph.byKind("class").firstOrNull { it.name == "x" }
    assertNull(
      phantom,
      "companion object body must not produce a class symbol; got: ${graph.byKind("class")}",
    )
    val property = graph.findByName("x").firstOrNull { it.kind == "property" }
    assertNotNull(property, "expected property symbol `x` in companion body")
  }

  @Test
  fun anonymous_object_expression_does_not_bind_a_stray_class_name() {
    // Regression: anonymous `object : Runnable { override fun run() = Unit }`
    // must not assign the body identifier `run` to the surrounding object.
    val src =
      """
      fun make(): Runnable = object : Runnable {
          override fun run() = Unit
      }
      """.trimIndent()
    val graph = parser.parse(src)
    val phantom = graph.byKind("class").firstOrNull { it.name == "run" }
    assertNull(
      phantom,
      "anonymous object body must not become a class symbol; got: ${graph.byKind("class")}",
    )
  }
}
