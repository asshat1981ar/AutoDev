//! Repository context selection for agentic software engineering.
//!
//! The context fabric turns a repository snapshot into a bounded, ranked set of
//! files suitable for a model request. It deliberately starts with deterministic
//! lexical retrieval instead of requiring an embedding service. This gives the
//! orchestrator a cheap local-first exploration primitive while leaving room for
//! an embedding/reranker backend later.
//!
//! The design follows a useful agent boundary: exploration produces evidence;
//! the model decides what that evidence means. The selector never edits files.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// A repository file candidate presented to the context selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextFile {
    /// Workspace-relative path.
    pub path: String,
    /// File contents used for lexical matching and eventual model context.
    pub content: String,
}

/// A ranked context item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextItem {
    pub path: String,
    /// Deterministic relevance score. Higher is better.
    pub score: u32,
    /// Why the file was selected.
    pub reasons: Vec<String>,
    /// Original content.
    pub content: String,
}

/// A bounded context pack ready for a model request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPack {
    pub query: String,
    pub items: Vec<ContextItem>,
    /// Total UTF-8 bytes in selected content.
    pub total_bytes: usize,
}

/// Configuration for deterministic repository retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPolicy {
    /// Maximum number of files selected.
    pub max_files: usize,
    /// Maximum UTF-8 bytes of selected file content.
    pub max_bytes: usize,
}

impl Default for ContextPolicy {
    fn default() -> Self {
        Self {
            max_files: 12,
            max_bytes: 64 * 1024,
        }
    }
}

/// Select the most relevant repository files for a natural-language task.
///
/// Scoring combines query-token matches, path matches, filename matches, and
/// common source/documentation signals. Ties are resolved by path, making the
/// result reproducible across runs and machines.
pub fn select_context(
    files: &[ContextFile],
    query: &str,
    policy: &ContextPolicy,
) -> ContextPack {
    let query_tokens = tokenize(query);
    let mut ranked: Vec<ContextItem> = files
        .iter()
        .map(|file| score_file(file, &query_tokens))
        .filter(|item| item.score > 0)
        .collect();

    ranked.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));

    let mut items = Vec::new();
    let mut total_bytes = 0usize;
    for item in ranked {
        if items.len() >= policy.max_files {
            break;
        }
        if item.content.len() > policy.max_bytes.saturating_sub(total_bytes) {
            continue;
        }
        total_bytes += item.content.len();
        items.push(item);
        if total_bytes >= policy.max_bytes {
            break;
        }
    }

    ContextPack {
        query: query.to_string(),
        items,
        total_bytes,
    }
}

fn score_file(file: &ContextFile, query_tokens: &BTreeSet<String>) -> ContextItem {
    let path_lower = file.path.to_ascii_lowercase();
    let filename = file
        .path
        .rsplit('/')
        .next()
        .unwrap_or(&file.path)
        .to_ascii_lowercase();
    let content_lower = file.content.to_ascii_lowercase();

    let mut score = 0u32;
    let mut reasons = Vec::new();

    for token in query_tokens {
        if filename.contains(token) {
            score += 10;
            reasons.push(format!("filename:{token}"));
        } else if path_lower.contains(token) {
            score += 6;
            reasons.push(format!("path:{token}"));
        }

        let occurrences = content_lower.matches(token).count().min(8) as u32;
        if occurrences > 0 {
            score += occurrences * 2;
            reasons.push(format!("content:{token}x{occurrences}"));
        }
    }

    // Give the model useful structural anchors when the query is broad.
    if is_source_file(&file.path) {
        score += 1;
        reasons.push("source".to_string());
    }
    if is_project_manifest(&file.path) {
        score += 3;
        reasons.push("manifest".to_string());
    }
    if is_test_file(&file.path) {
        score += 2;
        reasons.push("test".to_string());
    }

    ContextItem {
        path: file.path.clone(),
        score,
        reasons,
        content: file.content.clone(),
    }
}

fn tokenize(input: &str) -> BTreeSet<String> {
    let stop = [
        "a", "an", "and", "are", "for", "from", "how", "in", "into", "is", "of", "on",
        "or", "the", "to", "with", "this", "that", "it", "make", "add", "fix",
    ];
    input
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter_map(|raw| {
            let token = raw.to_ascii_lowercase();
            if token.len() < 2 || stop.contains(&token.as_str()) {
                None
            } else {
                Some(token)
            }
        })
        .collect()
}

fn is_source_file(path: &str) -> bool {
    matches!(
        path.rsplit('.').next().unwrap_or_default().to_ascii_lowercase().as_str(),
        "rs" | "kt" | "kts" | "go" | "ts" | "tsx" | "js" | "jsx" | "py" | "java" | "c" | "cpp" | "h"
    )
}

fn is_project_manifest(path: &str) -> bool {
    matches!(
        path.rsplit('/').next().unwrap_or(path),
        "Cargo.toml" | "package.json" | "pyproject.toml" | "go.mod" | "build.gradle" | "settings.gradle"
    )
}

fn is_test_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("/test") || lower.contains("tests/") || lower.ends_with("_test.rs") || lower.ends_with("_test.go")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, content: &str) -> ContextFile {
        ContextFile {
            path: path.into(),
            content: content.into(),
        }
    }

    #[test]
    fn ranks_filename_and_content_matches() {
        let files = vec![
            file("src/model.rs", "pub struct ModelProvider {}"),
            file("src/git.rs", "git workspace operations"),
        ];
        let pack = select_context(&files, "model provider", &ContextPolicy::default());
        assert_eq!(pack.items[0].path, "src/model.rs");
        assert!(pack.items[0].reasons.iter().any(|r| r == "filename:model"));
    }

    #[test]
    fn respects_file_and_byte_limits() {
        let files = vec![
            file("src/a.rs", "alpha alpha"),
            file("src/b.rs", "alpha beta"),
            file("src/c.rs", "alpha gamma"),
        ];
        let policy = ContextPolicy {
            max_files: 2,
            max_bytes: 12,
        };
        let pack = select_context(&files, "alpha", &policy);
        assert!(pack.items.len() <= 2);
        assert!(pack.total_bytes <= 12);
    }

    #[test]
    fn selection_is_deterministic_for_ties() {
        let files = vec![file("src/z.rs", "query"), file("src/a.rs", "query")];
        let pack = select_context(&files, "query", &ContextPolicy::default());
        assert_eq!(pack.items[0].path, "src/a.rs");
    }

    #[test]
    fn ignores_stop_words() {
        let files = vec![file("src/a.rs", "the and with")];
        let pack = select_context(&files, "the and with", &ContextPolicy::default());
        assert!(pack.items.is_empty());
    }
}
