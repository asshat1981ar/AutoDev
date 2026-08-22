package dev.autodev.codegraph

/**
 * A queryable symbol index built over an [AstSymbolGraph].
 *
 * Supports filtered lookups (by kind, enclosing scope) and the resolution of
 * an offset to its containing declaration. This is the query surface a control
 * plane consumes; extraction is done by [KotlinTreeSitterParser] (in mpp-core).
 */
public class SymbolGraphQueryEngine(private val graph: AstSymbolGraph) {
  /**
   * All symbols, optionally filtered by [kind].
   */
  public fun declarations(kind: String? = null): List<AstSymbol> {
    val all = graph.all()
    return if (kind == null) all else all.filter { it.kind == kind }
  }

  /**
   * Symbols declared directly inside [scope] (a dotted qualified name).
   */
  public fun membersOf(scope: String): List<AstSymbol> =
    graph.all().filter { it.enclosing == scope }

  /**
   * Resolve [offset] to the narrowest containing declaration.
   */
  public fun resolve(offset: Int): AstSymbol? = graph.symbolAt(offset)

  /**
   * Whether any symbol matches [name] exactly.
   */
  public fun declares(name: String): Boolean = graph.findByName(name).isNotEmpty()

  /**
   * A summary of the indexed unit.
   */
  public fun summary(): Summary =
    Summary(
      totalSymbols = graph.size(),
      functions = graph.byKind("function").size,
      classes = graph.byKind("class").size,
      properties = graph.byKind("property").size,
    )

  public data class Summary(
    val totalSymbols: Int,
    val functions: Int,
    val classes: Int,
    val properties: Int,
  )
}
