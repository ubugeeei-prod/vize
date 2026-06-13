//! vue/valid-v-text
//!
//! Enforce valid `v-text` directives.
//!
//! `v-text` must have an expression and must not have an argument or a
//! modifier.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-text></div>
//! <div v-text:arg="foo"></div>
//! <div v-text.mod="foo"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-text="msg"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-text",
    description: "Enforce valid `v-text` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-text directives
pub struct ValidVText;

impl Rule for ValidVText {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "text" {
            return;
        }

        if !has_expression(&directive.exp) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-text.missing_expression"),
                &directive.loc,
                ctx.t("vue/valid-v-text.help"),
            );
        }

        if directive.arg.is_some() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-text.unexpected_argument"),
                &directive.loc,
                ctx.t("vue/valid-v-text.help"),
            );
        }

        if !directive.modifiers.is_empty() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-text.unexpected_modifier"),
                &directive.loc,
                ctx.t("vue/valid-v-text.help"),
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
    use super::ValidVText;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVText));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text="msg"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_text_missing_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_text_with_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text:foo="msg"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_text_with_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-text.foo="msg"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
