package dev.autodev.mcp

/**
 * A typed tool that can be invoked through the MCP (Model Context Protocol)
 * JSON-RPC dispatcher.
 *
 * @property name     The method name, e.g. "tools/call".
 * @property handler  Maps the JSON params object to a JSON result.
 */
public class McpTool(
    public val name: String,
    private val handler: (Map<String, Any?>) -> Any?,
) {
    public fun invoke(params: Map<String, Any?>): Any? = handler(params)
}

/**
 * Result of dispatching a JSON-RPC request.
 */
public sealed class McpDispatchResult {
    /**
     * A successful response carrying the JSON-serializable result.
     */
    public data class Ok(val id: Any?, val result: Any?) : McpDispatchResult()

    /**
     * A JSON-RPC error with a code and message.
     */
    public data class Error(val id: Any?, val code: Int, val message: String) : McpDispatchResult()
}

/**
 * A minimal, dependency-free JSON-RPC 2.0 method dispatcher for MCP tools.
 *
 * Tools are registered by method name; [handle] parses a JSON-RPC envelope,
 * dispatches to the matching tool, and returns a structured result that callers
 * can serialize. Unknown methods and malformed requests map to standard
 * JSON-RPC errors (-32601 method not found, -32600 invalid request).
 */
public class McpToolHandler {
    private val tools: MutableMap<String, McpTool> = mutableMapOf()

    /**
     * Register a tool under [tool.name]. A later registration replaces an
     * earlier one.
     */
    public fun register(tool: McpTool): McpToolHandler {
        tools[tool.name] = tool
        return this
    }

    /**
     * Register a tool by name + handler.
     */
    public fun register(
        name: String,
        handler: (Map<String, Any?>) -> Any?,
    ): McpToolHandler = register(McpTool(name, handler))

    /**
     * Whether a tool is registered for [name].
     */
    public fun supports(name: String): Boolean = name in tools

    /**
     * Dispatch a parsed JSON-RPC request object.
     *
     * Expected shape: `{ "jsonrpc": "2.0", "id": ..., "method": "...", "params": {...} }`.
     */
    public fun handle(
        method: String,
        params: Map<String, Any?>,
    ): Any? {
        val tool =
            tools[method]
                ?: throw McpException(-32601, "method not found: $method")
        return tool.invoke(params)
    }

    /**
     * Dispatch a full JSON-RPC envelope map. Returns a structured result so
     * the caller can build the wire response without this class knowing the
     * JSON serializer.
     */
    public fun dispatch(request: Map<String, Any?>): McpDispatchResult {
        val id = request["id"]
        val rpc = request["jsonrpc"] as? String
        if (rpc != "2.0") {
            return McpDispatchResult.Error(id, -32600, "invalid request: jsonrpc must be \"2.0\"")
        }
        val method =
            request["method"] as? String
                ?: return McpDispatchResult.Error(id, -32600, "invalid request: missing method")

        @Suppress("UNCHECKED_CAST")
        val params = (request["params"] as? Map<String, Any?>) ?: emptyMap()
        return try {
            McpDispatchResult.Ok(id, handle(method, params))
        } catch (e: McpException) {
            McpDispatchResult.Error(id, e.code, e.message ?: "error")
        } catch (e: Throwable) {
            McpDispatchResult.Error(id, -32603, "internal error: ${e.message}")
        }
    }
}

/**
 * A JSON-RPC error raised by tool dispatch.
 */
public class McpException(val code: Int, message: String) : Exception(message)
