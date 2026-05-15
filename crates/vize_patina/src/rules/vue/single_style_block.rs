//! vue/single-style-block
//!
//! Recommend having a single style block in Vue SFCs.
//!
//! Multiple style blocks can make CSS harder to organize and maintain.
//! Using a single style block encourages better structure and makes
//! it easier to understand the component's styling.
//!
//! ## Exceptions
//!
//! This rule does not warn when style blocks have different purposes:
//! - One scoped and one global style block
//!
//! ## Examples
//!
//! Bad:
//! ```vue
//! <style scoped>
//! .component { color: red; }
//! </style>
//!
//! <style scoped>
//! .other { color: blue; }
//! </style>
//! ```
//!
//! Good:
//! ```vue
//! <style scoped>
//! .component { color: red; }
//! .other { color: blue; }
//! </style>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{parse_sfc, SfcParseOptions};
use vize_carton::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/single-style-block",
    description: "Recommend having a single style block",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

static HELP_MERGE_STYLES: &str =
    "Multiple style blocks of the same type can be merged for better organization";

#[derive(Debug, Clone, Copy)]
struct StyleBlock {
    start: u32,
    end: u32,
    scoped: bool,
}

/// Single style block rule.
#[derive(Default)]
pub struct SingleStyleBlock;

impl Rule for SingleStyleBlock {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let descriptor = match profile!(
            "patina.rule.single_style_block.parse_sfc",
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

        if descriptor.styles.len() <= 1 {
            return;
        }

        let mut style_blocks = Vec::with_capacity(descriptor.styles.len());
        for style in &descriptor.styles {
            style_blocks.push(StyleBlock {
                start: style.loc.tag_start as u32,
                end: style.loc.tag_end as u32,
                scoped: style.scoped,
            });
        }

        let all_scoped = style_blocks.iter().all(|style| style.scoped);
        let all_non_scoped = style_blocks.iter().all(|style| !style.scoped);

        if !(all_scoped || all_non_scoped) {
            return;
        }

        for style in style_blocks.iter().skip(1) {
            ctx.report(
                LintDiagnostic::warn(
                    META.name,
                    "Consider merging multiple style blocks into one",
                    style.start,
                    style.end,
                )
                .with_help(HELP_MERGE_STYLES),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SingleStyleBlock;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(SingleStyleBlock));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_single_style_block_is_valid() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div /></template>
<style scoped>
.component { color: red; }
</style>"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_multiple_scoped_style_blocks_warn() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div /></template>
<style scoped>
.component { color: red; }
</style>
<style scoped>
.other { color: blue; }
</style>"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/single-style-block");
    }

    #[test]
    fn test_multiple_global_style_blocks_warn() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div /></template>
<style>
.component { color: red; }
</style>
<style>
.other { color: blue; }
</style>"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_scoped_and_global_style_blocks_are_allowed() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div /></template>
<style>
body { margin: 0; }
</style>
<style scoped>
.component { color: red; }
</style>"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
