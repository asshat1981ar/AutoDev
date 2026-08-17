package dev.autodev.review

/**
 * Severity of a review finding.
 */
public enum class ReviewSeverity { Info, Warning, Error }

/**
 * A single review finding produced by comparing an original source to a
 * patched source.
 */
public data class ReviewFinding(
    val severity: ReviewSeverity,
    val message: String,
)

/**
 * Reviews a patched source against the original, surfacing structural
 * findings without executing the patch.
 *
 * The reviewer is deterministic and conservative: it flags no-op patches,
 * unmatched brace deltas, blank-line-only edits, and obvious whitespace-only
 * changes. It does not perform semantic analysis.
 */
public class AstPatchReviewer {
    /**
     * @return ordered list of findings; empty when the patch is clean.
     */
    public fun review(
        original: String,
        patched: String,
    ): List<ReviewFinding> {
        val findings = mutableListOf<ReviewFinding>()

        if (original == patched) {
            findings += ReviewFinding(ReviewSeverity.Warning, "no changes detected")
            return findings
        }

        if (original.replace(Regex("\\s+"), "") == patched.replace(Regex("\\s+"), "")) {
            findings +=
                ReviewFinding(
                    ReviewSeverity.Info,
                    "patch is whitespace-only",
                )
        }

        val origBraces = original.count { it == '{' } - original.count { it == '}' }
        val patchedBraces = patched.count { it == '{' } - patched.count { it == '}' }
        if (origBraces != patchedBraces) {
            findings +=
                ReviewFinding(
                    ReviewSeverity.Error,
                    "brace balance changed: open-minus-close was $origBraces, now $patchedBraces",
                )
        }

        val origParens = original.count { it == '(' } - original.count { it == ')' }
        val patchedParens = patched.count { it == '(' } - patched.count { it == ')' }
        if (origParens != patchedParens) {
            findings +=
                ReviewFinding(
                    ReviewSeverity.Error,
                    "parenthesis balance changed: was $origParens, now $patchedParens",
                )
        }

        val addedLines = patched.lineCount() - original.lineCount()
        if (addedLines != 0) {
            findings +=
                ReviewFinding(
                    ReviewSeverity.Info,
                    "line count changed by ${if (addedLines > 0) "+" else ""}$addedLines",
                )
        }

        return findings
    }

    private fun String.lineCount(): Int = if (isEmpty()) 0 else this.count { it == '\n' } + 1
}
