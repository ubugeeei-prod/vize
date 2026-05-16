//! vue/valid-v-bind
//!
//! Enforce valid `v-bind` directives.
//!
//! `v-bind` must:
//! - Have an attribute name (argument) or be used for object binding
//! - Have an expression (or use Vue 3.4+ same-name shorthand)
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-bind></div>
//! <div :></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div :class="foo"></div>
//! <div v-bind:class="foo"></div>
//! <div v-bind="{ class: foo }"></div>
//! <div :loading></div>  <!-- Vue 3.4+ same-name shorthand for :loading="loading" -->
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-bind",
    description: "Enforce valid `v-bind` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-bind directives
pub struct ValidVBind;

impl Rule for ValidVBind {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "bind" {
            return;
        }

        let has_arg = directive.arg.is_some();
        let has_exp = directive
            .exp
            .as_ref()
            .map(|e| !is_empty_expression(e))
            .unwrap_or(false);

        // Object syntax: v-bind="{ class: foo }"
        if !has_arg && has_exp {
            // This is valid object syntax
            return;
        }

        // Attribute syntax: :class="foo" or Vue 3.4+ same-name shorthand: :loading
        if has_arg {
            // Vue 3.4+ same-name shorthand allows :attr without expression
            // It's equivalent to :attr="attr"
            return;
        }

        // No argument and no expression
        ctx.error_with_help(
            ctx.t("vue/valid-v-bind.missing_expression"),
            &directive.loc,
            ctx.t("vue/valid-v-bind.help"),
        );
    }
}

/// Check if expression is empty
fn is_empty_expression(exp: &ExpressionNode) -> bool {
    match exp {
        ExpressionNode::Simple(s) => s.content.trim().is_empty(),
        ExpressionNode::Compound(c) => c.children.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::ValidVBind;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVBind));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_bind() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_long_form() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-bind:class="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_same_name_shorthand() {
        // Vue 3.4+ same-name shorthand: :loading is equivalent to :loading="loading"
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :loading></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_same_name_shorthand_multiple() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :loading :disabled :checked></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_bind_no_arg_no_exp() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-bind></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
