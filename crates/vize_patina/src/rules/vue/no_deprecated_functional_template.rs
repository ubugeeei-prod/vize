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
//! is a default correctness rule and only fires for the default Vue 3 dialect.
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
use vize_l0::dialect::VueDialect;
use vize_l0::profile;

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
        if ctx.dialect() != VueDialect::Vue || !ctx.is_rule_enabled(META.name) {
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

        // The SFC descriptor keeps the attribute's value but not its location.
        // Recover the complete attribute within the already parsed opening tag,
        // matching eslint-plugin-vue's VAttribute diagnostic target.
        let Some((start, end)) =
            functional_attribute_span(ctx.source, template.loc.tag_start, template.loc.start)
        else {
            return;
        };

        let message = ctx
            .t("vue/no-deprecated-functional-template.message")
            .into_owned();
        let help = ctx
            .t("vue/no-deprecated-functional-template.help")
            .into_owned();
        ctx.report(LintDiagnostic::error(META.name, message, start, end).with_help(help));
    }
}

fn functional_attribute_span(source: &str, start: usize, end: usize) -> Option<(u32, u32)> {
    let header = source.get(start..end)?.as_bytes();
    let mut cursor = 1;
    // Skip the opening tag's name, then inspect only complete attributes.
    while header
        .get(cursor)
        .is_some_and(|&byte| !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'/'))
    {
        cursor += 1;
    }
    while cursor < header.len() {
        while header.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if header
            .get(cursor)
            .is_none_or(|&byte| matches!(byte, b'>' | b'/'))
        {
            break;
        }
        let name_start = cursor;
        while header
            .get(cursor)
            .is_some_and(|&byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'>' | b'/'))
        {
            cursor += 1;
        }
        let name_end = cursor;
        if name_start == name_end {
            cursor += 1;
            continue;
        }
        while header.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let mut attribute_end = name_end;
        if header.get(cursor) == Some(&b'=') {
            cursor += 1;
            while header.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                cursor += 1;
            }
            if let Some(quote @ (b'\'' | b'"')) = header.get(cursor).copied() {
                cursor += 1;
                while header.get(cursor).is_some_and(|byte| *byte != quote) {
                    cursor += 1;
                }
                if header.get(cursor) != Some(&quote) {
                    return None;
                }
                cursor += 1;
            } else {
                while header.get(cursor).is_some_and(|&byte| {
                    !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'/')
                }) {
                    cursor += 1;
                }
            }
            attribute_end = cursor;
        }
        if header.get(name_start..name_end) == Some(b"functional") {
            return Some((
                u32::try_from(start + name_start).ok()?,
                u32::try_from(start + attribute_end).ok()?,
            ));
        }
    }
    None
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
