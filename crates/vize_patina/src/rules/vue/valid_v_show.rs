//! vue/valid-v-show
//!
//! Enforce valid `v-show` directives.
//!
//! `v-show` must:
//! - Have an expression
//! - Not be on `<template>` element
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-show></div>
//! <template v-show="foo"><div></div></template>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-show="foo"></div>
//! <div v-show="foo > 0"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-show",
    description: "Enforce valid `v-show` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-show directives
pub struct ValidVShow;

impl Rule for ValidVShow {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "show" {
            return;
        }

        // Check 1: v-show must have an expression
        let has_expression = match &directive.exp {
            Some(exp) => !is_empty_expression(exp),
            None => false,
        };

        if !has_expression {
            ctx.error_with_help(
                ctx.t("vue/valid-v-show.missing_expression"),
                &directive.loc,
                ctx.t("vue/valid-v-show.help"),
            );
            return;
        }

        // Check 2: v-show cannot be used on <template>
        if element.tag.as_str() == "template" {
            ctx.error_with_help(
                ctx.t("vue/valid-v-show.on_template"),
                &directive.loc,
                ctx.t("vue/valid-v-show.help"),
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
    use super::ValidVShow;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVShow));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_show() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-show="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_show_no_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-show></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_show_on_template() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-show="foo"><div></div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }
}
