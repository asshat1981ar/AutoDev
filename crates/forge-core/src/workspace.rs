//! Workspace confinement.
//!
//! The workspace is the trusted boundary between an agent's requested paths and
//! the host filesystem. Every path in an action payload is resolved and
//! validated against the workspace before any file operation occurs. This
//! defends against path traversal (`..`), absolute-path escapes, and symlink
//! escapes.

use std::path::{Component, Path, PathBuf};

/// The result of resolving a payload path against a workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathResolution {
    /// The path is a canonical path inside an allowed root.
    Allowed(PathBuf),
    /// The canonical path resolves outside the allowed roots.
    Denied(PathBuf),
    /// The path could not be interpreted (empty, malformed, traversal, etc.).
    Invalid(String),
}

/// A configured, allow-listed workspace root.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// The primary workspace root.
    root: PathBuf,
    /// Additional allowed roots (all canonicalized).
    allowed_roots: Vec<PathBuf>,
    /// Maximum number of bytes a single file may occupy.
    max_bytes: u64,
}

impl Workspace {
    /// Create a workspace rooted at `root` with the given size limit.
    ///
    /// The root is canonicalized eagerly so all later comparisons are against a
    /// stable, symlink-resolved base.
    pub fn new(root: impl AsRef<Path>, max_bytes: u64) -> std::io::Result<Self> {
        let mut root = root.as_ref().to_path_buf();
        if let Ok(canonical) = std::fs::canonicalize(&root) {
            root = canonical;
        }
        Ok(Workspace {
            allowed_roots: vec![root.clone()],
            root,
            max_bytes,
        })
    }

    /// The canonical primary root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The maximum file size, in bytes.
    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    /// Add an additional allowed root. Returns `false` if it cannot be
    /// canonicalized.
    pub fn add_allowed_root(&mut self, root: impl AsRef<Path>) -> bool {
        match std::fs::canonicalize(root) {
            Ok(canonical) => {
                self.allowed_roots.push(canonical);
                true
            }
            Err(_) => false,
        }
    }

    /// Resolve a raw payload path against the workspace.
    ///
    /// The raw path is interpreted as relative to the workspace root. It is
    /// made absolute, lexically normalized (resolving `.` and `..`), and then
    /// canonicalized (which resolves symlinks). The final canonical path must
    /// live inside one of the allowed roots.
    pub fn resolve_path(&self, raw: &Path) -> PathResolution {
        // Reject empty paths.
        if raw.as_os_str().is_empty() {
            return PathResolution::Invalid("empty path".to_string());
        }

        // If the path is absolute, resolve it directly; otherwise anchor it to
        // the workspace root.
        let anchored = if raw.is_absolute() {
            raw.to_path_buf()
        } else {
            self.root.join(raw)
        };

        // Normalize lexically (resolve `.` and `..`) before any comparison so
        // that `..` escapes cannot hide inside the literal path string.
        let normalized = normalize(&anchored);

        // Containment check on the normalized path.
        if !is_lexically_contained(&normalized, &self.root) {
            // A relative path that used `..` to escape is traversal; an
            // absolute path that simply points outside is a plain denial.
            if raw.components().any(|c| c == Component::ParentDir) {
                return PathResolution::Invalid(
                    "path escapes the workspace root (traversal)".to_string(),
                );
            }
            return PathResolution::Denied(anchored);
        }

        // Canonicalize to resolve symlinks; a symlink pointing outside the root
        // will canonicalize to a path outside the allowed roots.
        match std::fs::canonicalize(&normalized) {
            Ok(canonical) => {
                if self
                    .allowed_roots
                    .iter()
                    .any(|r| is_contained(&canonical, r))
                {
                    PathResolution::Allowed(canonical)
                } else {
                    PathResolution::Denied(canonical)
                }
            }
            Err(_) => {
                // The path does not yet exist or cannot be resolved; the
                // lexical containment check already passed, so allow it.
                PathResolution::Allowed(normalized)
            }
        }
    }
}

/// Whether `path` (assumed absolute) is lexically inside `root` (assumed
/// absolute). Does not resolve symlinks.
fn is_lexically_contained(path: &Path, root: &Path) -> bool {
    let mut current = Some(path);
    while let Some(p) = current {
        if p == root {
            return true;
        }
        current = p.parent();
    }
    false
}

/// Whether `path` (canonical) is inside `root` (canonical), component-wise.
fn is_contained(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
}

/// Lexically normalize an absolute path, removing `.` and resolving `..`
/// without touching the filesystem.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contained_relative_path_is_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 1024).unwrap();
        assert!(matches!(
            ws.resolve_path(Path::new("a/b.txt")),
            PathResolution::Allowed(_)
        ));
    }

    #[test]
    fn traversal_is_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path(), 1024).unwrap();
        assert!(matches!(
            ws.resolve_path(Path::new("../outside")),
            PathResolution::Invalid(_)
        ));
    }

    #[test]
    fn absolute_path_outside_root_is_denied() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("ws");
        std::fs::create_dir_all(&root).unwrap();
        let ws = Workspace::new(&root, 1024).unwrap();
        // A known absolute path outside the workspace.
        let outside = dir.path().join("secret.txt");
        assert!(matches!(
            ws.resolve_path(&outside),
            PathResolution::Denied(_)
        ));
    }

    #[test]
    fn symlink_escape_is_denied() {
        #[cfg(unix)]
        {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("ws");
            std::fs::create_dir_all(&root).unwrap();
            let outside = dir.path().join("secret.txt");
            std::fs::write(&outside, b"secret").unwrap();
            std::os::unix::fs::symlink(&outside, root.join("link.txt")).unwrap();
            let ws = Workspace::new(&root, 1024).unwrap();
            assert!(matches!(
                ws.resolve_path(Path::new("link.txt")),
                PathResolution::Denied(_)
            ));
        }
    }
}
