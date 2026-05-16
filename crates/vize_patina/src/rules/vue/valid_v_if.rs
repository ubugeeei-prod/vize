//! vue/valid-v-if
//!
//! Enforce valid `v-if` directives.
//!
//! `v-if` must:
//! - Have an expression
//! - Not be used with `v-else` or `v-else-if` on the same element
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-if></div>
//! <div v-if=""></div>
//! <div v-if="foo" v-else></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-if="foo"></div>
//! <div v-if="foo > 0"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-if",
    description: "Enforce valid `v-if` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-if directives
pub struct ValidVIf;

impl Rule for ValidVIf {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "if" {
            return;
        }

        // Check 1: v-if must have an expression
        let has_expression = match &directive.exp {
            Some(exp) => !is_empty_expression(exp),
            None => false,
        };

        if !has_expression {
            ctx.error_with_help(
                ctx.t("vue/valid-v-if.missing_expression"),
                &directive.loc,
                ctx.t("vue/valid-v-if.help"),
            );
        }

        // Check 2: v-if should not be used with v-else or v-else-if
        let has_v_else = element.props.iter().any(|p| {
            matches!(p, PropNode::Directive(d) if d.name.as_str() == "else" || d.name.as_str() == "else-if")
        });
        if has_v_else {
            ctx.error_with_help(
                ctx.t("vue/valid-v-if.missing_expression"),
                &directive.loc,
                ctx.t("vue/valid-v-if.help"),
            );
        }
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
    use super::ValidVIf;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVIf));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_if_with_comparison() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if="foo > 0"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_if_no_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_if_empty_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if=""></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
