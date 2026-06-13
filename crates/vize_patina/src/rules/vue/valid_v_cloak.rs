//! vue/valid-v-cloak
//!
//! Enforce valid `v-cloak` directives.
//!
//! `v-cloak` is a bare marker directive: it must not have an expression,
//! an argument, or a modifier.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-cloak="foo"></div>
//! <div v-cloak:arg></div>
//! <div v-cloak.mod></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-cloak></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-cloak",
    description: "Enforce valid `v-cloak` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-cloak directives
pub struct ValidVCloak;

impl Rule for ValidVCloak {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "cloak" {
            return;
        }

        if has_expression(&directive.exp) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-cloak.unexpected_value"),
                &directive.loc,
                ctx.t("vue/valid-v-cloak.help"),
            );
        }

        if directive.arg.is_some() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-cloak.unexpected_argument"),
                &directive.loc,
                ctx.t("vue/valid-v-cloak.help"),
            );
        }

        if !directive.modifiers.is_empty() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-cloak.unexpected_modifier"),
                &directive.loc,
                ctx.t("vue/valid-v-cloak.help"),
            );
        }
    }
}

/// Check whether the directive carries a non-empty expression.
fn has_expression(exp: &Option<ExpressionNode>) -> bool {
    match exp {
        Some(ExpressionNode::Simple(s)) => !s.content.trim().is_empty(),
        Some(ExpressionNode::Compound(c)) => !c.children.is_empty(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::ValidVCloak;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVCloak));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_cloak() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-cloak></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_cloak_with_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-cloak="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_cloak_with_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-cloak:foo></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_cloak_with_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-cloak.foo></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
