//! vue/valid-v-once
//!
//! Enforce valid `v-once` directives.
//!
//! `v-once` is a bare marker directive: it must not have an expression,
//! an argument, or a modifier.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-once="foo"></div>
//! <div v-once:arg></div>
//! <div v-once.mod></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-once></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-once",
    description: "Enforce valid `v-once` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-once directives
pub struct ValidVOnce;

impl Rule for ValidVOnce {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "once" {
            return;
        }

        if has_expression(&directive.exp) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-once.unexpected_value"),
                &directive.loc,
                ctx.t("vue/valid-v-once.help"),
            );
        }

        if directive.arg.is_some() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-once.unexpected_argument"),
                &directive.loc,
                ctx.t("vue/valid-v-once.help"),
            );
        }

        if !directive.modifiers.is_empty() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-once.unexpected_modifier"),
                &directive.loc,
                ctx.t("vue/valid-v-once.help"),
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
    use super::ValidVOnce;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVOnce));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_once() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-once></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_once_with_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-once="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_once_with_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-once:foo></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_once_with_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-once.foo></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
