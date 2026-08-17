package dev.autodev.codegraph

/**
 * Builds an [AstSymbolGraph] incrementally. Symbols are added in source order;
 * the resulting graph preserves insertion order.
 */
public class AstSymbolGraphBuilder {
    private val symbols: MutableList<AstSymbol> = mutableListOf()
    private var enclosingStack: ArrayDeque<String> = ArrayDeque()

    /**
     * Append a symbol. Returns this builder for chaining.
     */
    public fun add(symbol: AstSymbol): AstSymbolGraphBuilder {
        val resolved =
            if (symbol.enclosing == null && enclosingStack.isNotEmpty()) {
                symbol.copy(enclosing = enclosingStack.joinToString("."))
            } else {
                symbol
            }
        symbols.add(resolved)
        return this
    }

    /**
     * Push a containing scope name (e.g. when entering a class body) so that
     * subsequently added symbols are qualified by it.
     */
    public fun pushScope(scope: String): AstSymbolGraphBuilder {
        enclosingStack.addLast(scope)
        return this
    }

    /**
     * Pop the most recently pushed scope.
     */
    public fun popScope(): AstSymbolGraphBuilder {
        if (enclosingStack.isNotEmpty()) enclosingStack.removeLast()
        return this
    }

    /**
     * Freeze the builder into an immutable graph.
     */
    public fun build(): AstSymbolGraph = AstSymbolGraph(symbols.toList())
}
