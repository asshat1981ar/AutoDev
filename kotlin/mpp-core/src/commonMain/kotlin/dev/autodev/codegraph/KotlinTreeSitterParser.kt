package dev.autodev.codegraph

class KotlinTreeSitterParser {
    fun parse(source: String): AstSymbolGraph {
        // Placeholder: in production, integrate Tree-sitter and build symbol graph
        val builder = AstSymbolGraphBuilder()
        // naive placeholder: find "fun <name>" occurrences
        Regex("fun\\s+(\\w+)").findAll(source).forEachIndexed { idx, m ->
            val name = m.groupValues[1]
            builder.add(AstSymbol(name, "function", m.range))
        }
        return builder.build()
    }
}
