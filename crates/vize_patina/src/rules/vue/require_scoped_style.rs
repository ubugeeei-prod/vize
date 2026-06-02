//! vue/require-scoped-style
//!
//! Require `scoped` attribute on `<style>` tags.
//!
//! Scoped styles prevent CSS from leaking to other components and
//! make component styles more maintainable.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <style>
//! .button { color: red; }
//! </style>
//!
//! <style lang="scss">
//! .container { padding: 20px; }
//! </style>
//! ```
//!
//! ### Valid
//! ```vue
//! <style scoped>
//! .button { color: red; }
//! </style>
//!
//! <style scoped lang="scss">
//! .container { padding: 20px; }
//! </style>
//!
//! <!-- module styles are also fine -->
//! <style module>
//! .button { color: red; }
//! </style>
//! ```
//!
//! ## Exceptions
//!
//! - Global styles in App.vue or layout components may need unscoped styles
//! - CSS reset or normalize styles
//! - Deep selectors that need to affect child components

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/require-scoped-style",
    description: "Require scoped attribute on style tags",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require scoped style rule
#[derive(Default)]
pub struct RequireScopedStyle;

impl Rule for RequireScopedStyle {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let owned_descriptor;
        let descriptor = if let Some(descriptor) = ctx.sfc_descriptor() {
            descriptor
        } else {
            owned_descriptor = match profile!(
                "patina.rule.require_scoped_style.parse_sfc",
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

        // App/layout files are common exceptions for global styles.
        let filename = ctx.filename;
        let is_exception = filename.ends_with("App.vue")
            || filename.contains("layout")
            || filename.contains("Layout");
        if is_exception {
            return;
        }

        let unscoped_style_ranges = descriptor
            .styles
            .iter()
            .filter(|style| !style.scoped && style.module.is_none())
            .map(|style| (style.loc.tag_start as u32, style.loc.start as u32))
            .collect::<Vec<_>>();

        for (start, end) in unscoped_style_ranges {
            ctx.report(
                LintDiagnostic::warn(
                    META.name,
                    "Style block should use `scoped` or `module` attribute to prevent CSS leaking",
                    start,
                    end,
                )
                .with_help("Add `scoped` attribute: `<style scoped>`"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RequireScopedStyle;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(RequireScopedStyle));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_invalid_unscoped_style() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div>Hello</div></template>
<style>
.button { color: red; }
</style>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/require-scoped-style");
    }

    #[test]
    fn test_valid_scoped_style() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div>Hello</div></template>
<style scoped>
.button { color: red; }
</style>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_module_style() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div>Hello</div></template>
<style module>
.button { color: red; }
</style>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_app_vue_is_exception() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div>Hello</div></template>
<style>
body { margin: 0; }
</style>
"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_ignores_style_string_literals() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<script setup lang="ts">
const ssrStyles = computed(() => `<style>${styles}</style>`);
</script>

<template><div>Hello</div></template>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_reports_real_style_block_once_with_style_string_literal() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<script setup lang="ts">
const ssrStyles = computed(() => `<style>${styles}</style>`);
</script>

<template><div>Hello</div></template>
<style>
.button { color: red; }
</style>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/require-scoped-style");
    }
}
