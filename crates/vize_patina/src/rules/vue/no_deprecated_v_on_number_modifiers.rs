//! vue/no-deprecated-v-on-number-modifiers
//!
//! Disallow numeric `keyCode` modifiers on `v-on` (deprecated and removed in
//! Vue 3).
//!
//! In Vue 2, you could bind a keyboard handler to a specific physical key by its
//! `keyCode` directly in the modifier, e.g. `@keyup.13="submit"` (13 is Enter)
//! or `@keyup.27="cancel"` (27 is Escape). Vue 3 removed numeric `keyCode`
//! modifiers entirely in favour of named keys derived from `KeyboardEvent.key`,
//! e.g. `@keyup.enter` and `@keyup.esc`. A lingering numeric modifier in a Vue 3
//! template is no longer interpreted as a key filter, so the handler fires for
//! every key (or not at all) instead of only the intended one.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-v-on-number-modifiers`.
//!
//! ## Dialect gating
//!
//! The rule fires only for the default Vue 3 dialect. petite-vue and the legacy
//! Vue 2 / 2.7 dialect still understand numeric `keyCode` modifiers, so neither
//! should be flagged here.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <input @keyup.13="submit" />
//! <input v-on:keyup.27="cancel" />
//! <input @keyup.13.stop="submit" />
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <input @keyup.enter="submit" />
//! <input @keyup.esc="cancel" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-v-on-number-modifiers",
    description: "Disallow deprecated numeric `keyCode` modifiers on `v-on`",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow deprecated numeric `keyCode` modifiers on `v-on`.
pub struct NoDeprecatedVOnNumberModifiers;

impl Rule for NoDeprecatedVOnNumberModifiers {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only the default Vue 3 dialect removed numeric `keyCode` modifiers.
        // petite-vue and the legacy Vue 2 dialect still understand them, so
        // leave them alone.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        // Only `v-on` / `@` carries key-filter modifiers.
        if directive.name.as_str() != "on" {
            return;
        }

        // A numeric `keyCode` modifier is a modifier whose content is entirely
        // ASCII digits (e.g. "13", "27"). Report each one on its own location.
        for modifier in directive.modifiers.iter() {
            let content = modifier.content.as_str();
            if !content.is_empty() && content.bytes().all(|b| b.is_ascii_digit()) {
                ctx.error_with_help(
                    ctx.t_fmt(
                        "vue/no-deprecated-v-on-number-modifiers.message",
                        &[("keyCode", content)],
                    ),
                    &modifier.loc,
                    ctx.t("vue/no-deprecated-v-on-number-modifiers.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedVOnNumberModifiers;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedVOnNumberModifiers));
        Linter::with_registry(registry)
    }

    /// Wrap markup in a petite-vue document so `ctx.dialect()` resolves to
    /// petite-vue and the rule gates itself off.
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

    #[test]
    fn reports_number_modifier_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input @keyup.13="submit" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_number_modifier_full_syntax() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input v-on:keyup.27="cancel" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_number_combined_with_other_modifiers() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input @keyup.13.stop="submit" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_each_number_modifier() {
        // Two numeric modifiers on one directive should each be flagged.
        let linter = create_linter();
        let result = linter.lint_template(r#"<input @keyup.13.27="handler" />"#, "App.vue");
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn allows_named_key_modifiers() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<input @keyup.enter="submit" @keyup.esc="cancel" />"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_alphanumeric_modifier() {
        // A modifier that merely contains digits but is not purely numeric must
        // not be flagged (e.g. an `f2`-style named key).
        let linter = create_linter();
        let result = linter.lint_template(r#"<input @keyup.f2="handler" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_number_modifier_in_petite_vue() {
        // petite-vue still understands numeric keyCode modifiers; never flag it
        // there.
        let linter = create_linter();
        let result =
            linter.lint_standalone_html(&petite_doc(r#"<input @keyup.13="submit">"#), "index.html");
        assert_eq!(result.error_count, 0);
    }
}
