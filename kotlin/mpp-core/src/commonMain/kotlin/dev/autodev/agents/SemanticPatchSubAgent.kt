package dev.autodev.agents

import dev.autodev.codegraph.AstSymbolGraph
import dev.autodev.codegraph.KotlinTreeSitterParser
import dev.autodev.review.AstPatchReviewer
import dev.autodev.review.ReviewFinding

/**
 * A sub-agent that analyzes a source patch semantically: it extracts the
 * symbol graph of both the original and patched sources, then runs an
 * [AstPatchReviewer] over them, and reports the delta in declared symbols.
 *
 * It never executes the patch; it only inspects source text.
 */
public class SemanticPatchSubAgent {
    private val parser: KotlinTreeSitterParser = KotlinTreeSitterParser()
    private val reviewer: AstPatchReviewer = AstPatchReviewer()

    /**
     * The full analysis result.
     */
    public data class Analysis(
        val originalSymbols: AstSymbolGraph,
        val patchedSymbols: AstSymbolGraph,
        val addedSymbols: List<String>,
        val removedSymbols: List<String>,
        val reviewFindings: List<ReviewFinding>,
    )

    /**
     * Analyze the transition from [original] to [patched].
     */
    public fun analyze(
        original: String,
        patched: String,
    ): Analysis {
        val originalSymbols = parser.parse(original)
        val patchedSymbols = parser.parse(patched)

        val before = originalSymbols.all().map { it.qualifiedName }.toSet()
        val after = patchedSymbols.all().map { it.qualifiedName }.toSet()
        val added = (after - before).sorted()
        val removed = (before - after).sorted()

        val findings = reviewer.review(original, patched)
        return Analysis(
            originalSymbols = originalSymbols,
            patchedSymbols = patchedSymbols,
            addedSymbols = added,
            removedSymbols = removed,
            reviewFindings = findings,
        )
    }
}
