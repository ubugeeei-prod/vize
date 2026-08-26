//! vue/warn-custom-block
//!
//! Warn about custom blocks in SFC files.
//!
//! Custom blocks (blocks other than `<script>`, `<template>`, `<style>`)
//! require additional configuration and tooling support. This rule warns
//! about their usage to ensure they are intentional.
//!
//! ## Common Custom Blocks
//!
//! - `<i18n>` - Vue I18n translations
//! - `<docs>` - Component documentation
//! - `<story>` - Storybook stories
//! - `<test>` - Component tests
//!
//! ## Examples
//!
//! ### Triggers Warning
//! ```vue
//! <i18n>
//! { "en": { "hello": "Hello" } }
//! </i18n>
//!
//! <docs>
//! # MyComponent
//! This is a custom component.
//! </docs>
//! ```
//!
//! ## Detection
//!
//! Custom blocks are *top-level* SFC blocks (siblings of `<template>`,
//! `<script>`, and `<style>`), so detection relies on the SFC parser's block
//! boundaries rather than any raw-text or indentation heuristic. Elements
//! inside the `<template>` block — including unindented multi-root fragment
//! children starting at column 0 — are template content, never custom blocks
//! (issue #3210).
//!
//! ## Note
//!
//! This rule is informational. Custom blocks are valid and useful when
//! properly configured with the appropriate Vite/Webpack plugins.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s0::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/warn-custom-block",
    description: "Warn about custom blocks in SFC files",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Warn about custom blocks
#[derive(Default)]
pub struct WarnCustomBlock;

impl Rule for WarnCustomBlock {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        // Custom blocks are an SFC concept (`<i18n>`, `<docs>`, etc.).
        // Standalone HTML files (e.g. `index.html`, `.storybook/preview-head.html`)
        // are not Vue SFCs, so every top-level non-`script`/`template`/`style`
        // tag (`<link>`, `<meta>`, `<html>`, ...) would be flagged as a custom
        // block. Skip the rule on non-SFC files. See issue #2245.
        if !is_sfc_filename(ctx.filename) {
            return;
        }

        // The engine shares one parsed descriptor across SFC-level rules; parse
        // lazily only when a host drives this rule without preparing one. A
        // source that fails to parse as an SFC has no trustworthy block
        // boundaries — parse errors are reported elsewhere, so stay silent
        // instead of guessing.
        let owned_descriptor;
        let descriptor = if let Some(descriptor) = ctx.sfc_descriptor() {
            descriptor
        } else {
            owned_descriptor = match profile!(
                "patina.rule.warn_custom_block.parse_sfc",
                parse_sfc(
                    ctx.source,
                    SfcParseOptions {
                        filename: ctx.filename.into(),
                        ..Default::default()
                    },
                )
            ) {
                Ok(descriptor) => descriptor,
                Err(_) => return,
            };
            &owned_descriptor
        };

        if descriptor.custom_blocks.is_empty() {
            return;
        }

        // `loc.tag_start` is the `<` of the opening tag and `loc.start` is the
        // first content byte right past its `>` (past `/>` for self-closing
        // blocks), so this span covers exactly the opening tag. Collected
        // before reporting because the descriptor borrow must end before
        // `ctx.report` takes `ctx` mutably.
        let spans: Vec<(u32, u32)> = descriptor
            .custom_blocks
            .iter()
            .map(|block| (block.loc.tag_start as u32, block.loc.start as u32))
            .collect();

        for (start, end) in spans {
            ctx.report(
                LintDiagnostic::warn(
                    META.name,
                    "Custom block detected. Ensure proper plugin configuration.",
                    start,
                    end,
                )
                .with_help(
                    "Custom blocks require corresponding Vite/Webpack plugins to be processed",
                ),
            );
        }
    }
}

/// Returns `true` when the file should be treated as a Vue SFC for the purposes
/// of custom-block detection (i.e. its extension is `.vue`).
fn is_sfc_filename(filename: &str) -> bool {
    filename.rsplit('.').next() == Some("vue")
}

#[cfg(test)]
mod tests {
    use super::{WarnCustomBlock, is_sfc_filename};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(WarnCustomBlock));
        Linter::with_registry(registry)
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
        // The diagnostic must cover exactly the `<docs>` opening tag.
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
}

#[cfg(test)]
mod preset_tests {
    use crate::{LintPreset, Linter};

    fn warn_custom_block_count(result: &crate::LintResult) -> usize {
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/warn-custom-block")
            .count()
    }

    #[test]
    fn test_opinionated_preset_allows_root_fragment_issue_3210() {
        // The full preset exercises the shared-descriptor pipeline
        // (`lint_with_descriptor`), where template rules only ever see the
        // template inner content; the custom-block scan must not run there.
        let linter = Linter::with_preset(LintPreset::Opinionated);
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
        let linter = Linter::with_preset(LintPreset::Opinionated);
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
}
