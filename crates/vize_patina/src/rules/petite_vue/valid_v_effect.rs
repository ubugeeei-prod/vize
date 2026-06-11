//! petite-vue/valid-v-effect
//!
//! Require `v-effect` to carry a non-empty expression.
//!
//! petite-vue's `v-effect` runs its value as a reactive side-effect statement
//! that re-executes whenever the reactive state it reads changes. A `v-effect`
//! with no value (`v-effect`) or an empty value (`v-effect=""`) registers an
//! effect that does nothing, which is almost always an authoring mistake: the
//! author either forgot the statement or left a placeholder behind.
//!
//! This rule only runs on documents detected as petite-vue (see
//! `ctx.is_petite_vue()`); it has zero effect on normal Vue SFC linting, where
//! `v-effect` is not a directive at all.
//!
//! ## Examples
//!
//! ### Invalid (petite-vue document)
//! ```html
//! <div v-effect></div>
//! <div v-effect=""></div>
//! <div v-effect="   "></div>
//! ```
//!
//! ### Valid (petite-vue document)
//! ```html
//! <div v-effect="el.textContent = count"></div>
//! <div v-effect="count++"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "petite-vue/valid-v-effect",
    description: "Require v-effect to have a non-empty expression",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Require `v-effect` to have a non-empty expression.
#[derive(Default)]
pub struct ValidVEffect;

impl Rule for ValidVEffect {
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

        if directive.name != "effect" {
            return;
        }

        // `v-effect` with a value parses into an expression node; bare
        // `v-effect` leaves `exp` as `None`. A present-but-blank value
        // (`v-effect=""` / `v-effect="  "`) yields whitespace-only content.
        let is_empty = match &directive.exp {
            None => true,
            Some(ExpressionNode::Simple(exp)) => exp.content.trim().is_empty(),
            Some(ExpressionNode::Compound(_)) => false,
        };

        if !is_empty {
            return;
        }

        ctx.error_with_help(
            ctx.t("petite-vue/valid-v-effect.message"),
            &directive.loc,
            ctx.t("petite-vue/valid-v-effect.help"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::ValidVEffect;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVEffect));
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
    fn reports_bare_v_effect_in_petite_vue() {
        let linter = create_linter();
        let result =
            linter.lint_standalone_html(&petite_doc(r#"<div v-effect></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_empty_string_v_effect_in_petite_vue() {
        let linter = create_linter();
        let result =
            linter.lint_standalone_html(&petite_doc(r#"<div v-effect=""></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_whitespace_only_v_effect_in_petite_vue() {
        let linter = create_linter();
        let result =
            linter.lint_standalone_html(&petite_doc(r#"<div v-effect="   "></div>"#), "index.html");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_non_empty_v_effect_in_petite_vue() {
        let linter = create_linter();
        let markup = r#"<div v-effect="el.textContent = count"></div>
<span v-effect="count++"></span>"#;
        let result = linter.lint_standalone_html(&petite_doc(markup), "index.html");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_empty_v_effect_in_non_petite_document() {
        let linter = create_linter();
        // The same empty v-effect in a plain Vue document must not be flagged:
        // this rule is petite-vue-only and has zero effect on normal Vue.
        let result =
            linter.lint_standalone_html(&vue_doc(r#"<div v-effect=""></div>"#), "index.html");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_empty_v_effect_in_sfc_template() {
        let linter = create_linter();
        // A bare SFC template fragment is not a petite-vue document.
        let result = linter.lint_template(r#"<div v-effect=""></div>"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }
}
