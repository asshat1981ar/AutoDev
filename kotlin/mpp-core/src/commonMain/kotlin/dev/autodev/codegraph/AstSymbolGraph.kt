package dev.autodev.codegraph

/**
 * A half-open source span [start, end) measured in UTF-16 code units,
 * matching Kotlin/JS string indexing. Used to anchor symbols to source text
 * without depending on a specific parser's coordinate system.
 */
public data class SourceSpan(val start: Int, val end: Int) {
    init {
        require(start in 0..end) { "span must satisfy 0 <= start <= end, got [$start, $end)" }
    }

    public operator fun contains(offset: Int): Boolean = offset in start until end

    public fun length(): Int = end - start
}

/**
 * A declaration or reference discovered in source text.
 *
 * @property name     The simple identifier.
 * @property kind     Structural kind: "function", "class", "property", "parameter".
 * @property span     Source span of the identifier.
 * @property enclosing Optional qualified scope, e.g. "com.example.Foo.bar".
 */
public data class AstSymbol(
    val name: String,
    val kind: String,
    val span: SourceSpan,
    val enclosing: String? = null,
) {
    /**
     * The fully-qualified name, joining the enclosing scope with [name].
     */
    public val qualifiedName: String
        get() = if (enclosing.isNullOrEmpty()) name else "$enclosing.$name"
}

/**
 * An immutable symbol graph over a source unit. Supports lookup by name and
 * containment queries. Construction is via [AstSymbolGraphBuilder].
 */
public class AstSymbolGraph internal constructor(
    private val symbols: List<AstSymbol>,
) {
    /**
     * Every symbol in declaration order.
     */
    public fun all(): List<AstSymbol> = symbols

    /**
     * Symbols whose simple [AstSymbol.name] equals [name].
     */
    public fun findByName(name: String): List<AstSymbol> = symbols.filter { it.name == name }

    /**
     * Symbols whose [AstSymbol.qualifiedName] equals [qualifiedName].
     */
    public fun findByQualifiedName(qualifiedName: String): List<AstSymbol> = symbols.filter { it.qualifiedName == qualifiedName }

    /**
     * The narrowest symbol whose span contains [offset], or null if none.
     * Used to resolve a caret position to a binding.
     */
    public fun symbolAt(offset: Int): AstSymbol? =
        symbols
            .filter { offset in it.span }
            .minByOrNull { it.span.length() }

    /**
     * All symbols whose kind matches [kind].
     */
    public fun byKind(kind: String): List<AstSymbol> = symbols.filter { it.kind == kind }

    public fun isEmpty(): Boolean = symbols.isEmpty()

    public fun size(): Int = symbols.size

    override fun toString(): String = "AstSymbolGraph(size=${symbols.size})"
}
