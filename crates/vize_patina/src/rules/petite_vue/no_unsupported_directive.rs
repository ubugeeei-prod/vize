//! petite-vue/no-unsupported-directive
//!
//! Flag Vue-3-only directives that petite-vue does not support.
//!
//! petite-vue ships a deliberately small runtime with a limited built-in
//! directive set. Templates authored for petite-vue that reach for Vue 3 SFC
//! directives (such as `v-memo` or `v-slot`) or custom directives will silently
//! do nothing at runtime, because petite-vue has no mechanism to resolve them.
//!
//! This rule only runs on documents detected as petite-vue (see
//! `ctx.is_petite_vue()`); it has zero effect on normal Vue SFC linting.
//!
//! ## Supported directive set
//!
//! petite-vue supports: `v-scope`, `v-effect`, `v-if` / `v-else` / `v-else-if`,
//! `v-for`, `v-show`, `v-html`, `v-text`, `v-model`, `v-bind` (`:`), `v-on`
//! (`@`), `v-once`, `v-pre`, and `v-cloak`. The `ref` special attribute is also
//! supported, but it is parsed as a plain attribute rather than a directive, so
//! it never reaches this rule.
//!
//! ## Examples
//!
//! ### Invalid (petite-vue document)
//! ```html
//! <div v-memo="[a, b]"></div>
//! <template v-slot:header></template>
//! <div v-my-directive></div>
//! ```
//!
//! ### Valid (petite-vue document)
//! ```html
//! <div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
//! <div v-if="ok" v-bind:title="title" @click="count++"></div>
//! ```

#![allow(clippy::disallowed_macros)]

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::{CompactString, ToCompactString};
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "petite-vue/no-unsupported-directive",
    description: "Disallow directives that petite-vue does not support",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Normalized directive names that petite-vue supports at runtime.
///
/// Names are the parser's normalized form (no `v-` prefix; `:`/`@` map to
/// `bind`/`on`). `ref` is intentionally absent: petite-vue supports it, but it
/// is parsed as a plain attribute, so it never reaches `check_directive`.
const SUPPORTED_DIRECTIVES: &[&str] = &[
    "scope", "effect", "if", "else", "else-if", "for", "show", "html", "text", "model", "bind",
    "on", "once", "pre", "cloak",
];

/// Disallow directives unsupported by petite-vue.
#[derive(Default)]
pub struct NoUnsupportedDirective;

impl Rule for NoUnsupportedDirective {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only active for petite-vue documents; normal Vue SFCs are untouched.
        if !ctx.is_petite_vue() {
            return;
        }

        let name = directive.name.as_str();
        if SUPPORTED_DIRECTIVES.contains(&name) {
            return;
        }

        // Prefer the raw authored name (e.g. `v-memo`, `#header`) for the
        // message; fall back to the normalized `v-name` form.
        let display: CompactString = match &directive.raw_name {
            Some(raw) => raw.to_compact_string(),
            None => format!("v-{name}").into(),
        };

        ctx.error_with_help(
            ctx.t_fmt(
                "petite-vue/no-unsupported-directive.message",
                &[("directive", &display)],
            ),
            &directive.loc,
            ctx.t("petite-vue/no-unsupported-directive.help"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::NoUnsupportedDirective;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoUnsupportedDirective));
        Linter::with_registry(registry)
    }

    /// Wrap markup in a petite-vue document so `ctx.is_petite_vue()` is true.
    fn petite_doc(markup: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
  <body>
    <div v-scope="{{ count: 0 }}">
{markup}
    </div>
    <script src="https://unpkg.com/petite-vue" init></script>
  </body>
</html>"#
        )
    }

    /// Wrap markup in a plain (non-petite) Vue-loaded document.
    fn vue_doc(markup: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
  <body>
    <div>
{markup}
    </div>
    <script src="https://unpkg.com/vue"></script>
  </body>
</html>"#
        )
    }

    #[test]
    fn reports_v_memo_in_petite_vue() {
        let linter = create_linter();
        let result = linter
            .lint_standalone_html(&petite_doc(r#"<div v-memo="[a, b]"></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_v_slot_in_petite_vue() {
        let linter = create_linter();
        let result = linter.lint_standalone_html(
            &petite_doc(r#"<template v-slot:header></template>"#),
            "index.html",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_slot_shorthand_in_petite_vue() {
        let linter = create_linter();
        let result = linter.lint_standalone_html(
            &petite_doc(r#"<template #header></template>"#),
            "index.html",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_custom_directive_in_petite_vue() {
        let linter = create_linter();
        let result =
            linter.lint_standalone_html(&petite_doc(r#"<div v-my-directive></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_supported_directives_in_petite_vue() {
        let linter = create_linter();
        let markup = r#"<div v-effect="el.textContent = count" v-bind:title="t" @click="count++" v-show="count > 0" v-once></div>
<template v-if="count"></template>
<span v-for="i in 3" v-html="i" v-text="i"></span>"#;
        let result = linter.lint_standalone_html(&petite_doc(markup), "index.html");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_v_memo_in_non_petite_document() {
        let linter = create_linter();
        // The same v-memo in a plain Vue document must not be flagged: this rule
        // is petite-vue-only and has zero effect on normal Vue linting.
        let result =
            linter.lint_standalone_html(&vue_doc(r#"<div v-memo="[a, b]"></div>"#), "index.html");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_v_memo_in_sfc_template() {
        let linter = create_linter();
        // A bare SFC template fragment is not a petite-vue document.
        let result = linter.lint_template(r#"<div v-memo="[a, b]"></div>"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }
}
