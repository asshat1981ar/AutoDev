//! A deterministic, structured patch engine inspired by the unified diff
//! format used by GNU `patch` and `git apply`.
//!
//! The engine is intentionally **pure / in-memory**: it operates on
//! `Vec<String>` lines, so it is independently testable without touching the
//! filesystem or an autonomous agent. This mirrors the design rule that
//! ForgeCore's execution adapters are separated from orchestration.
//!
//! Two complementary capabilities are provided:
//!
//! - **Apply**: parse a unified diff and apply its hunks to source lines,
//!   validating context and detecting conflicts (like `git apply --check`).
//! - **Generate**: produce a unified diff from a before/after pair of line
//!   lists, used by the write executor to report the exact change it made.
//!
//! Design goals:
//!
//! - Prefer patches over whole-file replacement: a patch carries only the
//!   intended change plus surrounding context.
//! - Context validation: every hunk is anchored by context lines that must
//!   match the target exactly.
//! - Deterministic application and rollback: applying is a pure function; callers
//!   decide whether to persist the result.
//! - Conflict detection and failure reporting: stale context and overlapping
//!   edits are reported precisely, never silently applied.

/// The role of a line within a hunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// A context line (prefixed with a space in unified diff).
    Context,
    /// A line that exists in the target but not the result (prefixed `-`).
    Removed,
    /// A line that exists in the result but not the target (prefixed `+`).
    Added,
}

/// A single line of a hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchLine {
    pub kind: LineKind,
    /// The line content without its diff prefix.
    pub text: String,
}

/// A hunk: a contiguous edit anchored by a byte range and context lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchHunk {
    /// The 1-based start line in the original file.
    pub old_start: usize,
    /// The number of original lines this hunk spans.
    pub old_count: usize,
    /// The 1-based start line in the result file.
    pub new_start: usize,
    /// The number of result lines this hunk produces.
    pub new_count: usize,
    /// The hunk's lines (context, removed, added).
    pub lines: Vec<PatchLine>,
}

/// A parsed patch: a target filename and an ordered list of hunks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    /// The file the patch applies to (from the `+++` header).
    pub target_path: String,
    pub hunks: Vec<PatchHunk>,
}

/// The mode of a patch application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyMode {
    /// Validate and produce the result without persisting anything ("dry-run").
    Check,
    /// Produce the result to be persisted ("apply").
    Apply,
}

/// The outcome of applying a [`Patch`] to a source.
#[derive(Debug, Clone)]
pub struct PatchResult {
    /// The resulting lines, fully determined by the source + patch.
    pub new_lines: Vec<String>,
    /// The number of hunks successfully applied.
    pub applied_hunks: usize,
    /// The hunks that failed, with a reason.
    pub failures: Vec<PatchFailure>,
}

/// A hunk that could not be applied, with a structured reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchFailure {
    /// The hunk as parsed from the patch.
    pub hunk: PatchHunk,
    /// The reason it could not be applied.
    pub reason: PatchFailureReason,
}

/// Why a hunk failed to apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchFailureReason {
    /// The hunk header references a range outside the source file.
    RangeOutOfBounds {
        /// The 1-based start line.
        start: usize,
        /// The number of source lines spanned.
        count: usize,
        /// The number of source lines available.
        available: usize,
    },
    /// The hunk's anchor/context did not match the source at the expected
    /// location (stale context).
    StaleContext {
        /// The 1-based line where the hunk was expected.
        expected_line: usize,
        /// The text found at the expected location.
        found: Option<String>,
    },
    /// Two hunks in the same patch target overlapping regions of the source.
    Conflict {
        /// The 1-based line where the overlap occurred.
        line: usize,
    },
    /// The hunk's internal line counts do not sum to the declared ranges.
    MalformedCounts {
        /// The number of context/removed lines present.
        old_present: usize,
        /// The number of context/added lines present.
        new_present: usize,
    },
}

/// Errors that occur while parsing a patch document.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PatchParseError {
    /// The target file header (`+++`) is missing or malformed.
    #[error("missing or malformed target file header")]
    MissingTarget,
    /// A hunk header did not match the `@@ -a,b +c,d @@` form.
    #[error("malformed hunk header: {0}")]
    MalformedHunkHeader(String),
    /// A hunk referenced a zero or impossible range.
    #[error("invalid hunk range")]
    InvalidRange,
    /// The document contained a line that is valid in neither a header nor a
    /// hunk body.
    #[error("unexpected line: {0}")]
    UnexpectedLine(String),
}

impl Patch {
    /// Parse a unified-diff-style patch from `text`.
    ///
    /// Recognizes `--- a/...` / `+++ b/...` headers and `@@ -l,c +l,c @@`
    /// hunk headers followed by context (` `), removed (`-`), and added (`+`)
    /// lines. The target path is taken from the `+++` header.
    pub fn parse(text: &str) -> Result<Self, PatchParseError> {
        let mut target_path = None;
        let mut hunks = Vec::new();
        let mut lines = text.lines().peekable();

        while let Some(line) = lines.next() {
            if let Some(rest) = line.strip_prefix("+++ ") {
                target_path = Some(rest.trim().to_string());
                continue;
            }
            if let Some(rest) = line.strip_prefix("@@") {
                let ranges = parse_hunk_header(rest)?;
                // Collect the hunk body until the next header, without
                // consuming the header line (peek, don't advance).
                let mut body: Vec<String> = Vec::new();
                while let Some(next) = lines.peek() {
                    if next.starts_with("@@") || next.starts_with("+++") || next.starts_with("---")
                    {
                        break;
                    }
                    body.push(lines.next().unwrap().to_string());
                }
                let hunk = finish_hunk(ranges, &body)?;
                hunks.push(hunk);
            }
        }

        let target_path = target_path.ok_or(PatchParseError::MissingTarget)?;
        Ok(Patch { target_path, hunks })
    }
}

/// Split a unified-diff hunk header into its old/new ranges.
///
/// Accepts `@@ -l,c +l,c @@` and the `@@ -l +l @@` (count=1) shorthand.
fn parse_hunk_header(s: &str) -> Result<(usize, usize, usize, usize), PatchParseError> {
    // s looks like: " -12,4 +12,6 @@ ..." (the leading @@ was stripped).
    let rest = s.trim_start();
    let (old, new) = rest
        .split_once('+')
        .ok_or_else(|| PatchParseError::MalformedHunkHeader(s.to_string()))?;
    // The old range is prefixed with '-' (the diff's old-file marker).
    let old = old.trim().trim_start_matches('-');
    let new = new.trim();
    let (old_start, old_count) =
        parse_position(old).map_err(|_| PatchParseError::MalformedHunkHeader(s.to_string()))?;
    let (new_start, new_count) =
        parse_position(new).map_err(|_| PatchParseError::MalformedHunkHeader(s.to_string()))?;
    if old_start == 0 || new_start == 0 {
        return Err(PatchParseError::InvalidRange);
    }
    Ok((old_start, old_count, new_start, new_count))
}

/// Parse a `start,count` or `start` position (count defaults to 1).
fn parse_position(s: &str) -> Result<(usize, usize), ()> {
    let s = s.trim();
    // Some headers carry a trailing ` @@`; cut at the first space.
    let s = s.split_whitespace().next().unwrap_or(s);
    if let Some((a, b)) = s.split_once(',') {
        let start = a.parse::<usize>().map_err(|_| ())?;
        let count = b.parse::<usize>().map_err(|_| ())?;
        Ok((start, count))
    } else {
        let start = s.parse::<usize>().map_err(|_| ())?;
        Ok((start, 1))
    }
}

/// Build a [`PatchHunk`] from its header ranges and body lines.
///
/// Validates that the number of context/removed lines equals `old_count` and
/// the number of context/added lines equals `new_count`.
fn finish_hunk(
    (old_start, old_count, new_start, new_count): (usize, usize, usize, usize),
    body: &[String],
) -> Result<PatchHunk, PatchParseError> {
    let mut lines = Vec::with_capacity(body.len());
    let mut old_present = 0usize;
    let mut new_present = 0usize;
    for raw in body {
        match raw.as_bytes().first().copied() {
            Some(b' ') => {
                old_present += 1;
                new_present += 1;
                lines.push(PatchLine {
                    kind: LineKind::Context,
                    text: raw[1..].to_string(),
                });
            }
            Some(b'-') => {
                old_present += 1;
                lines.push(PatchLine {
                    kind: LineKind::Removed,
                    text: raw[1..].to_string(),
                });
            }
            Some(b'+') => {
                new_present += 1;
                lines.push(PatchLine {
                    kind: LineKind::Added,
                    text: raw[1..].to_string(),
                });
            }
            _ => return Err(PatchParseError::UnexpectedLine(raw.clone())),
        }
    }
    if old_present != old_count || new_present != new_count {
        return Err(PatchParseError::InvalidRange);
    }
    Ok(PatchHunk {
        old_start,
        old_count,
        new_start,
        new_count,
        lines,
    })
}
impl Patch {
    /// Apply this patch to `source` lines.
    ///
    /// Pure and deterministic: the result is a function of `source` and the
    /// patch. In [`ApplyMode::Check`] the caller treats the result as a dry-run
    /// (no persistence); in [`ApplyMode::Apply`] the caller may persist it.
    ///
    /// Hunks are applied in order. A hunk that cannot be applied is recorded in
    /// `failures` and does not stop later hunks, so partial failure is
    /// observable and reportable.
    pub fn apply(&self, source: &[String], _mode: ApplyMode) -> PatchResult {
        let mut new_lines: Vec<String> = source.to_vec();
        let mut applied = 0usize;
        let mut failures = Vec::new();

        // Running offset between the original file's line numbers and the
        // current buffer, in source lines (positive = buffer is longer).
        let mut offset: isize = 0;
        // Track which original lines have already been consumed by a prior hunk
        // so overlapping hunks are reported as conflicts.
        let mut consumed: Vec<bool> = vec![false; source.len().max(1)];

        for hunk in &self.hunks {
            if hunk.old_start == 0 || hunk.old_count == 0 {
                failures.push(PatchFailure {
                    hunk: hunk.clone(),
                    reason: PatchFailureReason::MalformedCounts {
                        old_present: 0,
                        new_present: 0,
                    },
                });
                continue;
            }

            // Position in the current buffer: original start + offset.
            let seek = hunk.old_start.saturating_sub(1) as isize + offset;
            if seek < 0 {
                failures.push(PatchFailure {
                    hunk: hunk.clone(),
                    reason: PatchFailureReason::RangeOutOfBounds {
                        start: hunk.old_start,
                        count: hunk.old_count,
                        available: new_lines.len(),
                    },
                });
                continue;
            }
            let start = seek as usize;

            // Verify there is room for old_count lines.
            if start + hunk.old_count > new_lines.len() {
                failures.push(PatchFailure {
                    hunk: hunk.clone(),
                    reason: PatchFailureReason::RangeOutOfBounds {
                        start: hunk.old_start,
                        count: hunk.old_count,
                        available: new_lines.len(),
                    },
                });
                continue;
            }

            // Detect overlap: any original line in this hunk already consumed?
            let orig_end = hunk.old_start + hunk.old_count; // exclusive, 1-based
            let overlapped =
                (hunk.old_start..orig_end).any(|ln| consumed[(ln - 1).min(consumed.len() - 1)]);
            if overlapped {
                failures.push(PatchFailure {
                    hunk: hunk.clone(),
                    reason: PatchFailureReason::Conflict {
                        line: hunk.old_start,
                    },
                });
                continue;
            }
            for ln in hunk.old_start..orig_end {
                if let Some(slot) = consumed.get_mut(ln - 1) {
                    *slot = true;
                }
            }

            // Match the hunk's context + removed lines at `start`.
            let mut matched = true;
            let mut found: Option<String> = None;
            let mut cursor = start;
            for line in &hunk.lines {
                match line.kind {
                    LineKind::Context | LineKind::Removed => {
                        if new_lines.get(cursor).map(String::as_str) != Some(line.text.as_str()) {
                            matched = false;
                            found = new_lines.get(cursor).cloned();
                            break;
                        }
                        cursor += 1;
                    }
                    LineKind::Added => {}
                }
            }
            if !matched {
                failures.push(PatchFailure {
                    hunk: hunk.clone(),
                    reason: PatchFailureReason::StaleContext {
                        expected_line: start + 1,
                        found,
                    },
                });
                continue;
            }

            // Build the replacement block: keep context + added lines.
            let block: Vec<String> = hunk
                .lines
                .iter()
                .filter(|l| l.kind != LineKind::Removed)
                .map(|l| l.text.clone())
                .collect();

            new_lines.splice(start..start + hunk.old_count, block.clone());
            applied += 1;
            offset += block.len() as isize - hunk.old_count as isize;
        }

        PatchResult {
            new_lines,
            applied_hunks: applied,
            failures,
        }
    }

    /// Dry-run: apply the patch and report whether it would succeed, without
    /// producing a result intended for persistence.
    pub fn apply_dry_run(&self, source: &[String]) -> PatchResult {
        self.apply(source, ApplyMode::Check)
    }

    /// Whether a patch result applied cleanly (no failures).
    pub fn is_clean(result: &PatchResult) -> bool {
        result.failures.is_empty()
    }
}
/// Generate a unified-diff string describing the change from `before` to
/// `after`.
///
/// This is a simple, deterministic line-based generator suitable for
/// reporting the change a write made (the "generated diff" evidence). It emits
/// a single hunk anchored at the first differing line, with adjacent
/// unchanged context lines. It is intentionally not a full Myers diff; it is
/// correct for the common cases of small, localized edits and always produces
/// a syntactically valid unified diff.
///
/// Returns `None` when `before == after` (no change).
pub fn generate_diff(before: &[String], after: &[String]) -> Option<String> {
    if before == after {
        return None;
    }

    // Find first differing line and last differing line (0-based).
    let first = before
        .iter()
        .zip(after.iter())
        .position(|(a, b)| a != b)
        .unwrap_or(before.len());
    let last = before
        .iter()
        .rev()
        .zip(after.iter().rev())
        .position(|(a, b)| a != b)
        .map(|i| before.len().saturating_sub(i))
        .unwrap_or(before.len());

    // Context window around the change.
    const CTX: usize = 2;
    let old_start0 = first.saturating_sub(CTX);
    let old_end0 = (last + CTX).min(before.len());
    let new_start0 = first.saturating_sub(CTX);
    let new_end0 = (last + CTX).min(after.len());

    let old_range_start = old_start0 + 1; // 1-based
    let old_count = old_end0 - old_start0;
    let new_range_start = new_start0 + 1;
    let new_count = new_end0 - new_start0;

    let mut lines = Vec::new();
    let mut i = old_start0;
    let mut j = new_start0;
    while i < old_end0 || j < new_end0 {
        let a = before.get(i);
        let b = after.get(j);
        match (a, b) {
            (Some(a), Some(b)) if a == b => {
                lines.push(format!(" {a}"));
                i += 1;
                j += 1;
            }
            (Some(a), Some(b)) if a != b => {
                lines.push(format!("-{a}"));
                lines.push(format!("+{b}"));
                i += 1;
                j += 1;
            }
            (Some(a), None) => {
                lines.push(format!("-{a}"));
                i += 1;
            }
            (None, Some(b)) => {
                lines.push(format!("+{b}"));
                j += 1;
            }
            (None, None) => break,
            // Unreachable: the two `Some/Some` arms above cover equality and
            // inequality.
            (Some(_), Some(_)) => break,
        }
    }

    let mut out = String::new();
    out.push_str("--- a/\n+++ b/\n");
    out.push_str(&format!(
        "@@ -{old_range_start},{old_count} +{new_range_start},{new_count} @@\n"
    ));
    for l in lines {
        out.push_str(&l);
        out.push('\n');
    }
    Some(out)
}
#[cfg(test)]
mod tests {
    use super::*;

    fn lines(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    const NORMAL: &str =
        "--- a/hello.txt\n+++ b/hello.txt\n@@ -1,3 +1,2 @@\n hello\n-old\n world\n";

    #[test]
    fn parse_sets_target_path() {
        let patch = Patch::parse(NORMAL).unwrap();
        assert_eq!(patch.target_path, "b/hello.txt");
        assert_eq!(patch.hunks.len(), 1);
    }

    #[test]
    fn normal_patch_applies() {
        let patch = Patch::parse(NORMAL).unwrap();
        let source = lines(&["hello", "old", "world"]);
        let result = patch.apply(&source, ApplyMode::Apply);
        assert!(Patch::is_clean(&result));
        assert_eq!(result.applied_hunks, 1);
        assert_eq!(result.new_lines, lines(&["hello", "world"]));
    }

    #[test]
    fn missing_target_is_an_error() {
        let err = Patch::parse("@@ -1,1 +1,1 @@\n a\n").unwrap_err();
        assert_eq!(err, PatchParseError::MissingTarget);
    }

    #[test]
    fn malformed_hunk_header_is_an_error() {
        let err = Patch::parse("--- a/f\n+++ b/f\n@@ nope @@\n").unwrap_err();
        assert!(matches!(err, PatchParseError::MalformedHunkHeader(_)));
    }

    #[test]
    fn stale_context_is_reported() {
        let patch = Patch::parse(NORMAL).unwrap();
        let source = lines(&["goodbye", "old", "world"]);
        let result = patch.apply(&source, ApplyMode::Apply);
        assert_eq!(result.applied_hunks, 0);
        assert_eq!(result.failures.len(), 1);
        assert!(matches!(
            result.failures[0].reason,
            PatchFailureReason::StaleContext { .. }
        ));
    }

    #[test]
    fn range_out_of_bounds_is_reported() {
        let patch = Patch::parse("--- a/f\n+++ b/f\n@@ -5,3 +5,3 @@\n x\n y\n z\n").unwrap();
        let source = lines(&["a", "b"]);
        let result = patch.apply(&source, ApplyMode::Apply);
        assert_eq!(result.applied_hunks, 0);
        assert!(matches!(
            result.failures[0].reason,
            PatchFailureReason::RangeOutOfBounds { .. }
        ));
    }

    #[test]
    fn dry_run_matches_apply_on_clean_input() {
        let patch = Patch::parse(NORMAL).unwrap();
        let source = lines(&["hello", "old", "world"]);
        let check = patch.apply_dry_run(&source);
        let apply = patch.apply(&source, ApplyMode::Apply);
        assert_eq!(check.new_lines, apply.new_lines);
        assert_eq!(check.applied_hunks, apply.applied_hunks);
        assert_eq!(check.failures, apply.failures);
    }

    #[test]
    fn multiple_hunks_apply_in_order() {
        let text = "--- a/f\n+++ b/f\n@@ -1,2 +1,1 @@\n a\n-old\n@@ -3,2 +3,1 @@\n c\n-old2\n";
        let patch = Patch::parse(text).unwrap();
        assert_eq!(patch.hunks.len(), 2);
        let source = lines(&["a", "old", "c", "old2"]);
        let result = patch.apply(&source, ApplyMode::Apply);
        assert!(Patch::is_clean(&result));
        assert_eq!(result.applied_hunks, 2);
        assert_eq!(result.new_lines, lines(&["a", "c"]));
    }

    #[test]
    fn conflicting_hunks_are_reported() {
        // Two hunks both target line 1.
        let text = "--- a/f\n+++ b/f\n@@ -1,1 +1,1 @@\n-a\n+b\n@@ -1,1 +1,1 @@\n-a\n+c\n";
        let patch = Patch::parse(text).unwrap();
        assert_eq!(patch.hunks.len(), 2);
        let source = lines(&["a"]);
        let result = patch.apply(&source, ApplyMode::Apply);
        assert_eq!(result.applied_hunks, 1);
        assert_eq!(result.failures.len(), 1);
        assert!(matches!(
            result.failures[0].reason,
            PatchFailureReason::Conflict { .. }
        ));
    }

    #[test]
    fn partial_failure_reports_succeeded_and_failed() {
        let text = "--- a/f\n+++ b/f\n@@ -1,2 +1,1 @@\n a\n-old\n@@ -9,2 +9,1 @@\n z\n-bad\n";
        let patch = Patch::parse(text).unwrap();
        let source = lines(&["a", "old", "b", "c", "d", "e", "f", "g", "h", "i"]);
        let result = patch.apply(&source, ApplyMode::Apply);
        assert_eq!(result.applied_hunks, 1);
        assert_eq!(result.failures.len(), 1);
        assert!(matches!(
            result.failures[0].reason,
            PatchFailureReason::StaleContext { .. }
        ));
    }

    #[test]
    fn malformed_counts_are_rejected_at_parse() {
        let text = "--- a/f\n+++ b/f\n@@ -1,3 +1,2 @@\n a\n-old\n";
        let err = Patch::parse(text);
        assert!(matches!(err, Err(PatchParseError::InvalidRange)));
    }

    #[test]
    fn generate_diff_emits_change() {
        let before = lines(&["a", "b", "c"]);
        let after = lines(&["a", "B", "c"]);
        let diff = generate_diff(&before, &after).unwrap();
        assert!(diff.contains("@@"));
        assert!(diff.contains("-b"));
        assert!(diff.contains("+B"));
    }

    #[test]
    fn generate_diff_none_when_equal() {
        let lines_a = lines(&["a", "b"]);
        assert!(generate_diff(&lines_a, &lines_a).is_none());
    }

    #[test]
    fn generate_diff_round_trips_through_apply() {
        let before = lines(&["one", "two", "three", "four"]);
        let after = lines(&["one", "TWO", "three", "four", "five"]);
        let diff = generate_diff(&before, &after).unwrap();
        let patch = Patch::parse(&diff).unwrap();
        let result = patch.apply(&before, ApplyMode::Apply);
        // The generated diff applies cleanly and reproduces `after`.
        assert_eq!(result.applied_hunks, patch.hunks.len());
        assert_eq!(result.new_lines, after);
    }
}
