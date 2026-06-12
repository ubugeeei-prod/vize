//! Building Vize expression nodes from OXC spans.

use oxc_ast::ast::{JSXExpression, JSXExpressionContainer};
use oxc_span::{GetSpan, Span};
use vize_carton::Box;
use vize_relief::{ExpressionNode, SimpleExpressionNode};

use super::Lowerer;

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// A dynamic (non-static) simple expression whose content is the source
    /// slice covered by `span`. Vize's later transform passes parse and prefix
    /// the identifiers; the lowering layer only needs the raw text + location.
    pub(crate) fn dyn_expr(&self, span: Span) -> ExpressionNode<'a> {
        let loc = self.mapper().location(span);
        let content = self.mapper().slice(span);
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(content, false, loc),
            self.bump(),
        ))
    }

    /// A static simple expression with explicit `content` at `span` (used for
    /// directive arguments and bound attribute names).
    pub(crate) fn static_expr(&self, content: &str, span: Span) -> ExpressionNode<'a> {
        let loc = self.mapper().location(span);
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(content, true, loc),
            self.bump(),
        ))
    }
}

/// The span of the expression inside a container, or `None` for an empty
/// container (`{}` / `{/* comment */}`).
pub(crate) fn container_expr_span(container: &JSXExpressionContainer<'_>) -> Option<Span> {
    match &container.expression {
        JSXExpression::EmptyExpression(_) => None,
        expression => Some(expression.span()),
    }
}
