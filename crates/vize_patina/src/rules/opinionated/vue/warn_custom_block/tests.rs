use super::{WarnCustomBlock, is_known_musea_art_block, is_sfc_filename};
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(WarnCustomBlock));
    Linter::with_registry(registry)
}

fn warn_custom_block_count(result: &crate::LintResult) -> usize {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/warn-custom-block")
        .count()
}

#[test]
fn detects_vue_sfc_filenames() {
    assert!(is_sfc_filename("Foo.vue"));
    assert!(is_sfc_filename("components/Foo.vue"));
    assert!(is_sfc_filename("/abs/path/App.vue"));
}

#[test]
fn rejects_standalone_html_and_other_filenames() {
    assert!(!is_sfc_filename("index.html"));
    assert!(!is_sfc_filename(".storybook/preview-head.html"));
    assert!(!is_sfc_filename("page.htm"));
    assert!(!is_sfc_filename("script.ts"));
    assert!(!is_sfc_filename("noext"));
}

#[test]
fn detects_musea_art_files() {
    assert!(is_known_musea_art_block("Button.art.vue", "art"));
    assert!(is_known_musea_art_block("catalog/Button.art.vue", "art"));
    assert!(!is_known_musea_art_block("Button.vue", "art"));
    assert!(!is_known_musea_art_block("Button.art.vue", "docs"));
}

#[test]
fn test_template_root_fragment_is_not_custom_block() {
    // Exact reproduction of issue #3210: `vize fmt --write` may leave
    // multi-root fragment children at column 0, which the old raw-text
    // scan mistook for top-level custom blocks.
    let linter = create_linter();
    let result = linter.lint_sfc(
        "<template>\n  <fieldset />\n<hr />\n<fieldset />\n</template>\n",
        "Component.vue",
    );
    assert_eq!(result.warning_count, 0, "got: {:?}", result.diagnostics);
}

#[test]
fn test_multi_root_template_with_mixed_indentation_is_valid() {
    // Multi-root fragments are valid regardless of indentation; children
    // at column 0, indented children, and nested unindented elements are
    // all inside the `<template>` block span.
    let linter = create_linter();
    let result = linter.lint_sfc(
        "<template>\n<header />\n    <main>\n<article />\n    </main>\n<custom-footer />\n</template>\n",
        "Component.vue",
    );
    assert_eq!(result.warning_count, 0, "got: {:?}", result.diagnostics);
}

#[test]
fn test_top_level_docs_block_warns() {
    let linter = create_linter();
    let source = "<template>\n  <div />\n</template>\n\n<docs>\n# MyComponent\n</docs>\n";
    let result = linter.lint_sfc(source, "Component.vue");
    assert_eq!(result.warning_count, 1, "got: {:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, "vue/warn-custom-block");
    let tag_start = source.find("<docs>").unwrap() as u32;
    assert_eq!(diagnostic.start, tag_start);
    assert_eq!(diagnostic.end, tag_start + "<docs>".len() as u32);
}

#[test]
fn test_top_level_i18n_block_warns() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        "<template>\n  <div />\n</template>\n<i18n>\n{ \"en\": { \"hello\": \"Hello\" } }\n</i18n>\n",
        "Component.vue",
    );
    assert_eq!(result.warning_count, 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].rule_name, "vue/warn-custom-block");
}

#[test]
fn test_musea_art_file_art_block_is_known() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        "<script setup lang=\"ts\">\ndefineArt(\"./Button.vue\", { title: \"Button\" })\n</script>\n<art>\n  <variant name=\"Default\" default>\n    <Button>OK</Button>\n  </variant>\n</art>\n",
        "Button.art.vue",
    );
    assert_eq!(result.warning_count, 0, "got: {:?}", result.diagnostics);
}

#[test]
fn test_art_block_in_regular_sfc_still_warns() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        "<template>\n  <div />\n</template>\n<art>\n  <variant name=\"Default\" />\n</art>\n",
        "Button.vue",
    );
    assert_eq!(result.warning_count, 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].rule_name, "vue/warn-custom-block");
}

#[test]
fn test_multiple_custom_blocks_warn_each() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        "<template>\n  <div />\n</template>\n<docs>\ntext\n</docs>\n<i18n>\n{}\n</i18n>\n",
        "Component.vue",
    );
    assert_eq!(result.warning_count, 2, "got: {:?}", result.diagnostics);
}

#[test]
fn test_self_closing_custom_block_with_src_warns() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        "<template>\n  <div />\n</template>\n<i18n src=\"./locales.json\" />\n",
        "Component.vue",
    );
    assert_eq!(result.warning_count, 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].rule_name, "vue/warn-custom-block");
}

#[test]
fn test_custom_block_content_at_column_zero_warns_once() {
    // Only the block itself is a custom block; markup inside its content
    // must not produce additional warnings even at column 0.
    let linter = create_linter();
    let result = linter.lint_sfc(
        "<template>\n  <div />\n</template>\n<docs>\n<hr />\ntext\n</docs>\n",
        "Component.vue",
    );
    assert_eq!(result.warning_count, 1, "got: {:?}", result.diagnostics);
}

#[test]
fn test_standard_blocks_only_do_not_warn() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        "<template>\n  <div />\n</template>\n<script setup>\nconst n = 1\n</script>\n<style scoped>\n.a { color: red; }\n</style>\n",
        "Component.vue",
    );
    assert_eq!(result.warning_count, 0, "got: {:?}", result.diagnostics);
}

#[test]
fn test_custom_block_in_script_only_sfc_warns() {
    // `run_on_sfc` fires without a template block, so custom blocks in
    // script-only components are still detected.
    let linter = create_linter();
    let result = linter.lint_sfc(
        "<script setup>\nconst n = 1\n</script>\n<story>\nDefault\n</story>\n",
        "Component.vue",
    );
    assert_eq!(result.warning_count, 1, "got: {:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].rule_name, "vue/warn-custom-block");
}

#[test]
fn test_opinionated_preset_allows_root_fragment_issue_3210() {
    // The full preset exercises the shared-descriptor pipeline
    // (`lint_with_descriptor`), where template rules only ever see the
    // template inner content; the custom-block scan must not run there.
    let linter = Linter::with_preset(crate::LintPreset::Opinionated);
    let result = linter.lint_sfc(
        "<template>\n  <fieldset />\n<hr />\n<fieldset />\n</template>\n",
        "FormLayout.vue",
    );
    assert_eq!(
        warn_custom_block_count(&result),
        0,
        "got: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_opinionated_preset_reports_top_level_custom_block() {
    let linter = Linter::with_preset(crate::LintPreset::Opinionated);
    let result = linter.lint_sfc(
        "<template>\n  <div />\n</template>\n<docs>\n# Docs\n</docs>\n",
        "FormLayout.vue",
    );
    assert_eq!(
        warn_custom_block_count(&result),
        1,
        "got: {:?}",
        result.diagnostics
    );
}
