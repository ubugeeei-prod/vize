//! Object, array and function literals in `v-bind` values.

use super::shared::static_argument;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::visitor::has_free_reference;
use oxc_ast::ast::Expression;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-literals-in-template",
    description: "Disallow object, array and function literals in v-bind",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-literals-in-template`.
pub struct NoLiteralsInTemplate;

impl Rule for NoLiteralsInTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "bind"
            || static_argument(directive)
                .is_some_and(|argument| matches!(argument, "class" | "style"))
        {
            return;
        }
        let Some(ExpressionNode::Simple(expression)) = &directive.exp else {
            return;
        };
        let Some(parsed) = expression.js_ast.and_then(|ast| ast.as_expression()) else {
            return;
        };
        // Like the reference, only the top-level expression counts, and a
        // literal reading a `v-for` or slot variable cannot move to script.
        if matches!(
            parsed.ast.without_parentheses(),
            Expression::ObjectExpression(_)
                | Expression::ArrayExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::ArrowFunctionExpression(_)
        ) && !(ctx.v_for_vars().next().is_some()
            && has_free_reference(expression.content, |name| ctx.is_v_for_var(name)))
        {
            ctx.warn_with_help(
                "Unexpected literal in a template binding",
                &expression.loc,
                "Move the literal to a script binding so Vue can reuse the same value",
            );
        }
    }
}
