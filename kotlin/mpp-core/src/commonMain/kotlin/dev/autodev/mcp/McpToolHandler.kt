package dev.autodev.mcp

class McpToolHandler {
    fun handle(method: String, params: Map<String, Any?>): Any? {
        // Minimal JSON-RPC handler stub
        return mapOf("ok" to true, "method" to method)
    }
}
