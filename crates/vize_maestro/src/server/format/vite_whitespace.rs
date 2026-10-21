//! Whole selection/on-type whitespace controls for the authored Vite policy.
use super::{format_on_type_with_template_whitespace, format_range_with_template_whitespace};
use tower_lsp::lsp_types::{Position, Range, TextEdit};

const SOURCE: &str = "<script setup>\nconst message=\"hello\"\n</script>\n<template>\n    <p>{{ message }}</p>\n</template>\n";

#[test]
fn preserve_range_keeps_template_text_and_default_retains_full_edit() {
    let options = vize_glyph::FormatOptions::default();
    let range = Range::new(Position::new(4, 0), Position::new(5, 0));
    let actual = |preserve| {
        format_range_with_template_whitespace(
            SOURCE,
            "App.vue",
            range,
            &options,
            vize_glyph::VueVersion::V3,
            preserve,
        )
    };
    assert_eq!(actual(true), Some(vec![]));
    assert_eq!(
        actual(false),
        Some(vec![TextEdit {
            range: Range::new(Position::new(3, 10), Position::new(5, 0)),
            new_text: "\n  <p>{{ message }}</p>\n".into(),
        }])
    );
}

#[test]
fn preserve_on_type_keeps_authored_template_indent_and_default_changes_only_indent() {
    let options = vize_glyph::FormatOptions::default();
    let actual = |preserve| {
        format_on_type_with_template_whitespace(
            SOURCE,
            "App.vue",
            Position::new(4, 4),
            &options,
            vize_glyph::VueVersion::V3,
            preserve,
        )
    };
    assert_eq!(actual(true), Some(vec![]));
    assert_eq!(
        actual(false),
        Some(vec![TextEdit {
            range: Range::new(Position::new(4, 0), Position::new(4, 4)),
            new_text: "  ".into(),
        }])
    );
}

#[test]
fn vite_compiler_whitespace_preserve_reaches_editor_formatter() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vite.config.mjs"),
        r#"export default { vue: {"compiler":{"whitespace":"preserve"},"formatter":{"singleQuote":true}} };"#,
    )
    .unwrap();
    let state = crate::server::ServerState::new();
    state.load_workspace_config(project.path());
    let (options, preserve) = state.get_formatter_context();
    assert_eq!((options.single_quote, preserve), (true, true));
}

#[test]
fn vue2_preserve_reaches_editor_formatter() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vite.config.mjs"),
        r#"export default { vue: { compiler: { whitespace: "vue2-preserve" } } };"#,
    )
    .unwrap();
    let state = crate::server::ServerState::new();
    state.load_workspace_config(project.path());

    assert!(
        state
            .get_formatter_context_for_path(&project.path().join("src/App.vue"))
            .1
    );
}

#[test]
fn entry_compiler_whitespace_is_resolved_for_each_editor_document() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vite.config.mjs"),
        r#"export default { vue: {
  compiler: { whitespace: "preserve" },
  entries: [
    { files: ["src/**"], compiler: { whitespace: "condense" } },
    { files: ["src/legacy/**"], compiler: { whitespace: "vue2-preserve" } }
  ]
} };"#,
    )
    .unwrap();
    let state = crate::server::ServerState::new();
    state.load_workspace_config(project.path());

    let preserves = |file| {
        state
            .get_formatter_context_for_path(&project.path().join(file))
            .1
    };
    assert!(!preserves("src/App.vue"));
    assert!(preserves("src/legacy/App.vue"));
    assert!(preserves("outside/App.vue"));
}
