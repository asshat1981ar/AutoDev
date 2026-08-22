package dev.autodev.mcp

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class McpToolHandlerTest {
  @Test
  fun dispatches_registered_tool() {
    val handler =
      McpToolHandler().register("tools/call") { params ->
        mapOf("echo" to params["name"])
      }
    val result =
      handler.dispatch(
        mapOf(
          "jsonrpc" to "2.0",
          "id" to 1,
          "method" to "tools/call",
          "params" to mapOf("name" to "hello"),
        ),
      )
    assertTrue(result is McpDispatchResult.Ok)
    assertEquals(1, result.id)
    assertEquals(mapOf("echo" to "hello"), result.result)
  }

  @Test
  fun unknown_method_returns_method_not_found() {
    val handler = McpToolHandler()
    val result =
      handler.dispatch(
        mapOf("jsonrpc" to "2.0", "id" to "2", "method" to "nope"),
      )
    assertTrue(result is McpDispatchResult.Error)
    assertEquals(-32601, result.code)
  }

  @Test
  fun invalid_jsonrpc_version_returns_invalid_request() {
    val handler = McpToolHandler()
    val result =
      handler.dispatch(
        mapOf("jsonrpc" to "1.0", "id" to 3, "method" to "x"),
      )
    assertTrue(result is McpDispatchResult.Error)
    assertEquals(-32600, result.code)
  }

  @Test
  fun handle_throws_for_unregistered_method() {
    val handler = McpToolHandler()
    assertFailsWith<McpException> {
      handler.handle("missing", emptyMap())
    }
  }

  @Test
  fun supports_reports_registered_tools() {
    val handler = McpToolHandler().register("ping") { "pong" }
    assertTrue(handler.supports("ping"))
    assertTrue(!handler.supports("absent"))
  }
}
