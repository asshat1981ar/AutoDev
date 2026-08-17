package dev.autodev.codegraph

class AstSymbolGraphBuilder {
    private val symbols = mutableListOf<AstSymbol>()

    fun add(symbol: AstSymbol): AstSymbolGraphBuilder {
        symbols.add(symbol)
        return this
    }

    fun build(): AstSymbolGraph = AstSymbolGraph(symbols.toList())
}
