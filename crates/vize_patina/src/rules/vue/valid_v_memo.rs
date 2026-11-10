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
use oxc_ast::ast::Expression;
use oxc_syntax::operator::LogicalOperator;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-memo",
    description: "Enforce valid `v-memo` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

#[derive(Default)]
/// Implements the `vue/valid-v-memo` lint rule.
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
    let Some(expression) = simple
        .js_ast
        .filter(|retained| retained.raw == simple.content)
        .and_then(|retained| retained.as_expression())
    else {
        return false;
    };
    expression_is_definitely_non_array(expression.ast)
}

fn expression_is_definitely_non_array(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::ClassExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::PrivateInExpression(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_) => true,
        Expression::LogicalExpression(logical) => match logical.operator {
            LogicalOperator::And => expression_is_definitely_non_array(&logical.right),
            LogicalOperator::Or | LogicalOperator::Coalesce => {
                expression_is_definitely_non_array(&logical.left)
                    && expression_is_definitely_non_array(&logical.right)
            }
        },
        Expression::AssignmentExpression(assignment) => {
            expression_is_definitely_non_array(&assignment.right)
        }
        Expression::ConditionalExpression(conditional) => {
            expression_is_definitely_non_array(&conditional.consequent)
                || expression_is_definitely_non_array(&conditional.alternate)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_is_definitely_non_array(&parenthesized.expression)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .is_some_and(expression_is_definitely_non_array),
        Expression::TSAsExpression(wrapper) => {
            expression_is_definitely_non_array(&wrapper.expression)
        }
        Expression::TSSatisfiesExpression(wrapper) => {
            expression_is_definitely_non_array(&wrapper.expression)
        }
        Expression::TSTypeAssertion(wrapper) => {
            expression_is_definitely_non_array(&wrapper.expression)
        }
        Expression::TSNonNullExpression(wrapper) => {
            expression_is_definitely_non_array(&wrapper.expression)
        }
        Expression::TSInstantiationExpression(wrapper) => {
            expression_is_definitely_non_array(&wrapper.expression)
        }
        _ => false,
    }
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
    fn rejects_non_array_ast_shapes_without_scanning_nested_expressions() {
        let linter = create_linter();
        for expression in [
            "item.id === selected",
            "!open",
            "1",
            "null",
            "true",
            "() => [selected]",
            "cache = 42",
        ] {
            let source = format!(r#"<div v-memo="{expression}" />"#);
            let result = linter.lint_template(&source, "test.vue");
            assert_eq!(result.error_count, 1, "{expression}");
        }

        for expression in [
            "getDeps(page - 1)",
            "rows.map((row) => row.id * 2)",
            "ok ? [a] : [b - 1]",
            "deps['data-id']",
        ] {
            let source = format!(r#"<div v-memo="{expression}" />"#);
            let result = linter.lint_template(&source, "test.vue");
            assert_eq!(result.error_count, 0, "{expression}");
        }
    }

    #[test]
    fn logical_fallbacks_are_rejected_only_when_every_result_is_non_array() {
        let linter = create_linter();
        for expression in ["deps || []", "deps ?? []", "ready && deps"] {
            let source = format!(r#"<div v-memo="{expression}" />"#);
            assert_eq!(linter.lint_template(&source, "test.vue").error_count, 0);
        }
        for expression in ["false || 1", "null ?? 'bad'", "ready && 1"] {
            let source = format!(r#"<div v-memo="{expression}" />"#);
            assert_eq!(linter.lint_template(&source, "test.vue").error_count, 1);
        }
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
