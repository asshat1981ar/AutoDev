package dev.autodev.codegraph

/**
 * A deterministic, dependency-free Kotlin source extractor that produces an
 * [AstSymbolGraph].
 *
 * This is a structural tokenizer rather than a full grammar: it recognizes the
 * declaration keywords (`fun`, `class`, `interface`, `object`, `val`, `var`,
 * `property`) and their associated identifiers, tracks brace nesting to assign
 * qualified scopes, and ignores string/char/comment bodies so that identifiers
 * inside literals are never treated as declarations.
 *
 * It is intentionally conservative — every emitted symbol corresponds to a
 * real declaration keyword — which makes the output stable and safe to use as
 * the query surface that a future tree-sitter-backed extractor can replace.
 */
public class KotlinTreeSitterParser {
  public constructor()

  /**
   * Parse [source] into a symbol graph.
   */
  public fun parse(source: String): AstSymbolGraph {
    val builder = AstSymbolGraphBuilder()
    val tokens = tokenize(source)
    var i = 0
    val scopeStack: ArrayDeque<String> = ArrayDeque()
    var braceDepth = 0

    while (i < tokens.size) {
      val tok = tokens[i]

      // Track brace depth so scopes are only popped at the right level.
      when (tok.kind) {
        TokenKind.LBrace -> {
          braceDepth += 1
          i += 1
          continue
        }
        TokenKind.RBrace -> {
          braceDepth -= 1
          if (braceDepth >= scopeStack.size) {
            // safety: never pop below zero
          } else if (scopeStack.isNotEmpty()) {
            scopeStack.removeLast()
            builder.popScope()
          }
          i += 1
          continue
        }
        else -> Unit
      }

      val (symbol, consumed) = tryExtractDeclaration(tokens, i, scopeStack)
      if (symbol != null && consumed > 0) {
        builder.add(symbol)
        // If the declaration opens a body, its name becomes a scope.
        if (symbol.kind in scopeOpeners) {
          scopeStack.addLast(symbol.name)
          builder.pushScope(symbol.name)
        }
        i += consumed
      } else {
        i += 1
      }
    }
    return builder.build()
  }

  private fun tryExtractDeclaration(
    tokens: List<Token>,
    i: Int,
    scopeStack: ArrayDeque<String>,
  ): Pair<AstSymbol?, Int> {
    val tok = tokens.getOrNull(i) ?: return null to 0
    val keyword = tok.keywordDeclaration() ?: return null to 0

    // Find the identifier that follows the declaration keyword. The scan
    // must only cross tokens that are legitimate decoration (annotations,
    // visibility/sync modifiers, generics) and must stop at anything that
    // is part of the body or a new syntactic construct. Otherwise, an
    // `object` declared as `companion object { val x = 1 }` would walk past
    // the opening brace and mis-classify `x` as a class named "x".
    val j = nextIdentifierAfterDeclaration(tokens, i + 1) ?: return null to 0
    val nameTok = tokens[j]
    val span = SourceSpan(start = nameTok.start, end = nameTok.start + nameTok.text.length)
    val enclosing =
      if (scopeStack.isEmpty()) null else scopeStack.joinToString(".")
    val symbol =
      AstSymbol(
        name = nameTok.text,
        kind = keyword.symbolKind,
        span = span,
        enclosing = enclosing,
      )
    return symbol to (j - i + 1)
  }

  /**
   * Return the index of the first Identifier token that is a legitimate
   * declaration name following `from`, skipping modifier keywords,
   * annotations, type parameters, and the `companion` keyword that
   * commonly appears before `object` / `val` / `var`. Returns null when
   * the declaration is anonymous or the tokens do not name one.
   */
  private fun nextIdentifierAfterDeclaration(
    tokens: List<Token>,
    from: Int,
  ): Int? {
    val modifierKeywords: Set<String> =
      setOf(
        "public", "private", "protected", "internal", "open", "abstract",
        "final", "sealed", "data", "enum", "annotation", "override",
        "lateinit", "suspend", "inline", "noinline", "crossinline",
        "expect", "actual", "external", "infix", "operator", "tailrec",
        "vararg", "const", "companion",
      )
    var j = from
    while (j < tokens.size) {
      val t = tokens[j]
      when {
        t.kind == TokenKind.Identifier -> return j
        t.kind == TokenKind.Keyword && t.text in modifierKeywords -> j += 1
        // Brace / paren / equals / comma / semicolon / other punctuation
        // ends the name search: we are now inside the declaration's body
        // or a different syntactic construct.
        t.kind == TokenKind.LBrace || t.kind == TokenKind.RBrace -> return null
        else -> return null
      }
    }
    return null
  }

  private enum class KeywordDeclaration(val symbolKind: String) {
    Fun("function"),
    Class("class"),
    Interface("class"),
    Object("class"),
    Val("property"),
    Var("property"),
    ;

    val keyword: String = name.lowercase()
  }

  private fun Token.keywordDeclaration(): KeywordDeclaration? =
    if (kind == TokenKind.Keyword) {
      KeywordDeclaration.entries.firstOrNull { it.keyword == text }
    } else {
      null
    }

  private companion object {
    val scopeOpeners = setOf("class", "interface", "object", "function")
  }

  // ---- Tokenizer ------------------------------------------------------

  private enum class TokenKind {
    Identifier,
    Keyword,
    LBrace,
    RBrace,
    Other,
  }

  private data class Token(val kind: TokenKind, val text: String, val start: Int)

  private val keywords: Set<String> =
    setOf(
      "fun", "class", "interface", "object", "val", "var",
      "private", "public", "protected", "internal", "open", "abstract",
      "final", "sealed", "data", "suspend", "override", "lateinit",
      "companion", "enum", "annotation",
    )

  private fun tokenize(source: String): List<Token> {
    val out = mutableListOf<Token>()
    val s = source
    val n = s.length
    var i = 0
    while (i < n) {
      val c = s[i]
      when {
        c.isWhitespace() -> i += 1

        // Line comment
        c == '/' && i + 1 < n && s[i + 1] == '/' -> {
          i += 2
          while (i < n && s[i] != '\n') i += 1
        }
        // Block comment
        c == '/' && i + 1 < n && s[i + 1] == '*' -> {
          i += 2
          while (i < n && !(s[i] == '*' && i + 1 < n && s[i + 1] == '/')) i += 1
          i += 2
        }
        // String literal (raw and normal)
        c == '"' -> {
          if (i + 2 < n && s[i + 1] == '"' && s[i + 2] == '"') {
            i += 3
            while (i < n &&
              !(
                s[i] == '"' && i + 2 < n &&
                  s[i + 1] == '"' && s[i + 2] == '"'
              )
            ) {
              if (s[i] == '\\' && i + 1 < n) i += 2 else i += 1
            }
            i += 3
          } else {
            i += 1
            while (i < n && s[i] != '"' && s[i] != '\n') {
              if (s[i] == '\\' && i + 1 < n) i += 2 else i += 1
            }
            if (i < n && s[i] == '"') i += 1
          }
        }
        // Char literal
        c == '\'' -> {
          i += 1
          while (i < n && s[i] != '\'' && s[i] != '\n') {
            if (s[i] == '\\' && i + 1 < n) i += 2 else i += 1
          }
          if (i < n && s[i] == '\'') i += 1
        }

        c == '{' -> {
          out.add(Token(TokenKind.LBrace, "{", i))
          i += 1
        }
        c == '}' -> {
          out.add(Token(TokenKind.RBrace, "}", i))
          i += 1
        }
        // Annotation / label start; skip the name.
        c == '@' -> {
          i += 1
          while (i < n && (s[i].isLetterOrDigit() || s[i] == '_' || s[i] == ':')) i += 1
        }

        c.isLetter() || c == '_' -> {
          val start = i
          while (i < n && (s[i].isLetterOrDigit() || s[i] == '_')) i += 1
          val text = s.substring(start, i)
          val kind =
            if (text in keywords) TokenKind.Keyword else TokenKind.Identifier
          out.add(Token(kind, text, start))
        }

        else -> i += 1
      }
    }
    return out
  }
}
