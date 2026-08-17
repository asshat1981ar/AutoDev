package dev.autodev.codegraph

data class AstSymbol(val name: String, val type: String, val range: IntRange)

class AstSymbolGraph(private val symbols: List<AstSymbol>) {
    fun findByName(name: String) = symbols.filter { it.name == name }
}
