//! vue/valid-v-memo
//!
//! Enforce valid `v-memo` directives.
//!
//! `v-memo` (Vue 3.2+) memoizes a sub-tree of the template. It requires:
//! - An expression (the dependency array)
//! - The expression should be an array
//! - When used with `v-for`, it must be placed on the same element
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-memo></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-memo="[valueA, valueB]">content</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-memo",
    description: "Enforce valid `v-memo` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

#[derive(Default)]
pub struct ValidVMemo;

impl Rule for ValidVMemo {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "memo" {
            return;
        }

        // v-memo must have an expression
        let has_expression = match &directive.exp {
            Some(exp) => match exp {
                ExpressionNode::Simple(s) => !s.content.trim().is_empty(),
                ExpressionNode::Compound(c) => !c.children.is_empty(),
            },
            None => false,
        };

        if !has_expression {
            ctx.error_with_help(
                ctx.t("vue/valid-v-memo.missing_expression"),
                &directive.loc,
                ctx.t("vue/valid-v-memo.help"),
            );
            return;
        }

        if ctx.has_ancestor(|ancestor| ancestor.has_v_for) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-memo.inside_v_for"),
                &directive.loc,
                ctx.t("vue/valid-v-memo.help"),
            );
        }

        if let Some(exp) = &directive.exp
            && is_definitely_non_array_expression(exp)
        {
            ctx.error_with_help(
                ctx.t("vue/valid-v-memo.expected_array"),
                &directive.loc,
                ctx.t("vue/valid-v-memo.help"),
            );
        }

        // v-memo should not have an argument
        if directive.arg.is_some() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-memo.unexpected_argument"),
                &directive.loc,
                ctx.t("vue/valid-v-memo.help"),
            );
        }

        // v-memo should not have modifiers
        if !directive.modifiers.is_empty() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-memo.unexpected_modifier"),
                &directive.loc,
                ctx.t("vue/valid-v-memo.help"),
            );
        }
    }
}

fn is_definitely_non_array_expression(exp: &ExpressionNode) -> bool {
    let ExpressionNode::Simple(simple) = exp else {
        return false;
    };
    let expr = simple.content.trim();
    if expr.starts_with('[') {
        return false;
    }
    expr.starts_with('"')
        || expr.starts_with('\'')
        || expr.starts_with('`')
        || expr.starts_with('{')
        || expr.starts_with("function")
        || expr.starts_with("class")
        || expr.ends_with("++")
        || expr.ends_with("--")
        || contains_binary_operator(expr)
}

fn contains_binary_operator(expr: &str) -> bool {
    ["+", "-", "*", "/", "%"]
        .iter()
        .any(|operator| expr.contains(operator))
}

#[cfg(test)]
mod tests {
    use super::ValidVMemo;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVMemo));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_memo_with_array() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-memo="[valueA, valueB]">content</div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_memo_with_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-memo="deps">content</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_memo_with_binary_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-memo="count + 1">content</div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_memo_with_update_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-memo="count++">content</div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_memo_inside_v_for() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id"><span v-memo="[item]"></span></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_v_memo_on_same_element_as_v_for() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id" v-memo="[item]"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_memo_no_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-memo>content</div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
