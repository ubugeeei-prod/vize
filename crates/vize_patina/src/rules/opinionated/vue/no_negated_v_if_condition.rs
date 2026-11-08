//! vue/no-negated-v-if-condition
//!
//! Disallow a negated `v-if` condition (`v-if="!x"`) when the same conditional
//! chain has a following `v-else`.
//!
//! When a `v-if` whose condition is a logical negation is paired with a
//! `v-else`, the two branches read backwards: the "happy path" lives in the
//! `v-else`. Swapping the condition (drop the `!`) and the branch bodies makes
//! the template easier to follow.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-if="!ok">A</div>
//! <div v-else>B</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-if="ok">A</div>
//! <div v-else>B</div>
//!
//! <div v-if="!ok">A</div>
//!
//! <div v-if="a !== b">A</div>
//! <div v-else>B</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use vize_armature::patterns::attribute_source_span;
use vize_relief::{
    ElementNode, ExpressionNode, PropNode, RootNode, SourceLocation, TemplateChildNode,
};

static META: RuleMeta = RuleMeta {
    name: "vue/no-negated-v-if-condition",
    description: "Disallow a negated v-if condition when the chain has a v-else",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow a negated `v-if` condition paired with a `v-else`.
pub struct NoNegatedVIfCondition;

impl Rule for NoNegatedVIfCondition {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        check_children(ctx, &root.children);
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_children(ctx, &element.children);
    }
}

/// Match eslint-plugin-vue's adjacent-pair semantics: only a negated `v-if` or
/// `v-else-if` immediately followed by the next element's `v-else` is reported.
fn check_children(ctx: &mut LintContext, children: &[TemplateChildNode]) {
    let mut previous_element: Option<&ElementNode> = None;
    for child in children.iter() {
        let TemplateChildNode::Element(el) = child else {
            continue;
        };
        if has_directive(el, "else")
            && let Some(expression) =
                previous_element.and_then(|element| negated_condition(element, ctx.source))
        {
            ctx.warn_with_help(
                ctx.t("vue/no-negated-v-if-condition.message"),
                &expression,
                ctx.t("vue/no-negated-v-if-condition.help"),
            );
        }
        previous_element = Some(el);
    }
}

fn has_directive(element: &ElementNode, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == name))
}

fn negated_condition(element: &ElementNode<'_>, source: &str) -> Option<SourceLocation> {
    element.props.iter().find_map(|prop| {
        let PropNode::Directive(directive) = prop else {
            return None;
        };
        if !matches!(directive.name, "if" | "else-if") {
            return None;
        }
        let expression = directive.exp.as_ref()?;
        expression_is_negated(expression).then(|| precise_expression_location(expression, source))
    })
}

fn precise_expression_location(expression: &ExpressionNode<'_>, source: &str) -> SourceLocation {
    let ExpressionNode::Simple(simple) = expression else {
        let loc = expression.loc();
        return SourceLocation::new(loc.span.start, loc.span.end);
    };
    let Some(parsed) = simple.js_ast.and_then(|parsed| parsed.as_expression()) else {
        return SourceLocation::new(simple.loc.span.start, simple.loc.span.end);
    };
    let authored = source
        .get(simple.loc.span.start as usize..simple.loc.span.end as usize)
        .unwrap_or_default();
    let span = parsed.ast.get_inner_expression().span();
    let Some(span) = attribute_source_span(authored, simple.content, span) else {
        return SourceLocation::new(simple.loc.span.start, simple.loc.span.end);
    };
    SourceLocation::new(
        simple.loc.span.start + span.start,
        simple.loc.span.start + span.end,
    )
}

fn expression_is_negated(exp: &ExpressionNode) -> bool {
    let ExpressionNode::Simple(simple) = exp else {
        return false;
    };
    if let Some(parsed) = simple.js_ast.and_then(|parsed| parsed.as_expression()) {
        return match parsed.ast.get_inner_expression() {
            Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::LogicalNot,
            Expression::BinaryExpression(binary) => matches!(
                binary.operator,
                BinaryOperator::Inequality | BinaryOperator::StrictInequality
            ),
            _ => false,
        };
    }
    let trimmed = simple.content.trim();
    (trimmed.starts_with('!') && !trimmed.starts_with("!="))
        || [" !== ", " != "]
            .iter()
            .any(|operator| trimmed.contains(operator))
}

#[cfg(test)]
mod tests {
    use super::NoNegatedVIfCondition;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoNegatedVIfCondition));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_negated_v_if_with_v_else() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div v-if="!ok">A</div><div v-else>B</div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_negated_v_if_before_else_if_then_else() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="!ok">A</div><div v-else-if="other">B</div><div v-else>C</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_negated_v_if_without_v_else() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if="!ok">A</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_negated_v_if_with_only_else_if() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="!ok">A</div><div v-else-if="other">B</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_plain_v_if_with_v_else() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div v-if="ok">A</div><div v-else>B</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_strict_inequality_condition() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="a !== b">A</div><div v-else>B</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_loose_inequality_condition() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="a != b">A</div><div v-else>B</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_negation_with_surrounding_whitespace() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="  !ok ">A</div><div v-else>B</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
        let diagnostic = &result.diagnostics[0];
        let expected =
            r#"<div v-if="  !ok ">A</div><div v-else>B</div>"#.find("!ok").unwrap() as u32;
        assert_eq!((diagnostic.start, diagnostic.end), (expected, expected + 3));
    }

    #[test]
    fn dangling_v_else_does_not_panic() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-else>B</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_negated_else_if_directly_followed_by_else() {
        let result = create_linter().lint_template(
            r#"<div v-if="first">A</div><div v-else-if="!second">B</div><div v-else>C</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn does_not_report_negated_if_before_else_if() {
        let result = create_linter().lint_template(
            r#"<div v-if="!first">A</div><div v-else-if="second">B</div><div v-else>C</div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
