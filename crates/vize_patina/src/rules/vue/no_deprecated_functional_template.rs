//! vue/no-deprecated-functional-template
//!
//! Disallow the `functional` attribute on the SFC `<template>` (removed in Vue
//! 3).
//!
//! Vue 2 let you author a functional component as a single-file component by
//! marking the template block functional: `<template functional>`. The render
//! output had no instance, props were read off a `props` context object, and the
//! component skipped the reactivity/lifecycle machinery. Vue 3 removed functional
//! single-file-component templates entirely: the performance gap that motivated
//! them closed, and a stateful component is now the one way to author a `.vue`
//! file. A lingering `<template functional>` no longer produces a functional
//! component — the `functional` attribute is simply ignored — so the migration
//! intent silently breaks. Functional components in Vue 3 are written as plain
//! functions (typically in JSX / a render function), not as SFC templates.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-functional-template`. It
//! is an opt-in migration rule and only fires for the default Vue 3 dialect.
//!
//! ## Scope
//!
//! eslint-plugin-vue reports the `functional` attribute on the SFC root
//! `<template>` element. patina runs markup rules and SFC-level rules in separate
//! passes, and the `<template>` SFC block's opening tag (with its attributes) is
//! not part of the template AST handed to markup rules — only the block's inner
//! content is. The `functional` attribute is, however, preserved on the parsed
//! SFC descriptor's template block, so this rule runs as an SFC-level rule
//! (`run_on_sfc`) over `descriptor.template.attrs`. That is exactly the SFC root
//! `<template>` element eslint-plugin-vue targets — a nested
//! `<template functional>` inside the markup is not valid SFC syntax and has no
//! analogue — so the SFC-descriptor check is both sound and complete for the
//! cases the original rule covers.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <template functional>
//!   <div>{{ props.msg }}</div>
//! </template>
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <template>
//!   <div>{{ msg }}</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::dialect::VueDialect;
use vize_carton::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-functional-template",
    description: "Disallow the `functional` attribute on the SFC `<template>`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the `functional` attribute on the SFC `<template>`.
#[derive(Default)]
pub struct NoDeprecatedFunctionalTemplate;

impl Rule for NoDeprecatedFunctionalTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        // Only the default Vue 3 dialect removed functional SFC templates. SFCs
        // are always the Vue dialect (petite-vue uses standalone HTML, never a
        // `.vue` file), so this guard is effectively always true here; it mirrors
        // the sibling migration rules and keeps the rule inert for any non-Vue
        // dialect a future caller might thread through.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        // Prefer the descriptor prepared by the engine; only parse the SFC
        // ourselves when one was not shared. `run_on_sfc` never runs for plain
        // template input (that path does not invoke SFC-level rules), so this rule
        // does nothing in that case.
        let owned_descriptor;
        let descriptor = if let Some(descriptor) = ctx.sfc_descriptor() {
            descriptor
        } else {
            owned_descriptor = match profile!(
                "patina.rule.no_deprecated_functional_template.parse_sfc",
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

        let Some(template) = descriptor.template.as_ref() else {
            return;
        };

        // The `functional` attribute is recorded on the template block's attribute
        // map (a boolean attribute keeps an empty value). Vue 2 only ever emitted
        // it lowercase, and HTML attribute names are ASCII case-insensitive, so an
        // exact `functional` match is both sufficient and free of false positives.
        if !template.attrs.contains_key("functional") {
            return;
        }

        // Highlight just the opening `<template ...>` tag: `tag_start` is the `<`
        // of the opening tag and `loc.start` is the content start (the byte right
        // after the opening tag's `>`), so this span is the opening tag exactly,
        // not the whole block.
        let start = template.loc.tag_start as u32;
        let end = template.loc.start as u32;

        let message = ctx
            .t("vue/no-deprecated-functional-template.message")
            .into_owned();
        let help = ctx
            .t("vue/no-deprecated-functional-template.help")
            .into_owned();
        ctx.report(LintDiagnostic::error(META.name, message, start, end).with_help(help));
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedFunctionalTemplate;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedFunctionalTemplate));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_functional_template() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            "<template functional>\n  <div>{{ props.msg }}</div>\n</template>\n",
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
        assert_eq!(
            result.diagnostics[0].rule_name,
            "vue/no-deprecated-functional-template"
        );
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_functional_template_with_other_attrs() {
        // `functional` combined with another block attribute (e.g. `lang`) is
        // still the removed functional template and must be flagged.
        let linter = create_linter();
        let result = linter.lint_sfc(
            "<template lang=\"html\" functional>\n  <div>{{ props.msg }}</div>\n</template>\n",
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_plain_template() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            "<template>\n  <div>{{ msg }}</div>\n</template>\n",
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_template_with_unrelated_attr() {
        // A non-`functional` block attribute (e.g. `lang`) must not be flagged.
        let linter = create_linter();
        let result = linter.lint_sfc(
            "<template lang=\"pug\">\n  div Hello\n</template>\n",
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn does_not_flag_functional_inside_markup() {
        // A nested `<div is="functional">`-style usage, or the word "functional"
        // appearing as content/attribute value inside the template body, is not
        // the SFC root `functional` attribute and must not be flagged.
        let linter = create_linter();
        let result = linter.lint_sfc(
            "<template>\n  <div data-kind=\"functional\">functional</div>\n</template>\n",
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn does_nothing_on_plain_template_input() {
        // Plain template input (not a full SFC) never runs SFC-level rules.
        let linter = create_linter();
        let result = linter.lint_template("<div>Hello</div>", "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn flags_only_once_with_script_block_present() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            "<template functional>\n  <div>{{ props.msg }}</div>\n</template>\n<script>\nexport default {};\n</script>\n",
            "App.vue",
        );
        assert_eq!(result.error_count, 1);
    }
}
