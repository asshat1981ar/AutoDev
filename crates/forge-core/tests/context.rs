use forge_core::{select_context, ContextFile, ContextPolicy};

#[test]
fn repository_context_prefers_relevant_source_files() {
    let files = vec![
        ContextFile {
            path: "src/model.rs".into(),
            content: "pub struct ModelProvider { /* routing */ }".into(),
        },
        ContextFile {
            path: "src/git.rs".into(),
            content: "git workspace operations".into(),
        },
        ContextFile {
            path: "README.md".into(),
            content: "AutoDev software engineering platform".into(),
        },
    ];

    let pack = select_context(
        &files,
        "improve model routing provider",
        &ContextPolicy {
            max_files: 2,
            max_bytes: 1024,
        },
    );

    assert_eq!(pack.items.len(), 1);
    assert_eq!(pack.items[0].path, "src/model.rs");
    assert!(pack.total_bytes <= 1024);
}

#[test]
fn repository_context_does_not_force_irrelevant_structural_files() {
    let files = vec![
        ContextFile {
            path: "src/runtime.rs".into(),
            content: "pub struct Runtime".into(),
        },
        ContextFile {
            path: "tests/runtime.rs".into(),
            content: "runtime tests".into(),
        },
    ];

    let pack = select_context(&files, "database migration", &ContextPolicy::default());
    assert!(pack.items.is_empty());
    assert_eq!(pack.total_bytes, 0);
}
