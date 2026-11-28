//! Immutable AST walks that lend nodes for the borrow of the walked root.
//!
//! The private generated walk follows the pinned OXC traversal, with every
//! node argument borrowed for `'a`. Descending through fields, arena vectors
//! and arena boxes preserves that borrow, so rustc proves that a hook may
//! retain a node for the root's lifetime. The markup facade and component-
//! options rules use those retained references after the walk returns.

use oxc_ast::ast::{
    CallExpression, Expression, JSXElement, JSXExpressionContainer, JSXFragment, Program,
};

mod generated;

#[cfg(test)]
mod tests;

/// The nodes a walk lends at the root's lifetime `'a`.
///
/// Every hook has a default that keeps oxc's default traversal.
pub(crate) trait LentNodes<'a> {
    /// A JSX element. Return whether to walk into it.
    #[inline]
    fn enter_jsx_element(&mut self, _element: &'a JSXElement<'a>) -> bool {
        true
    }

    /// Called after walking into an element whose enter hook returned true.
    #[inline]
    fn leave_jsx_element(&mut self, _element: &'a JSXElement<'a>) {}

    /// A JSX fragment. Return whether to walk into it.
    #[inline]
    fn enter_jsx_fragment(&mut self, _fragment: &'a JSXFragment<'a>) -> bool {
        true
    }

    /// Called after walking into a fragment whose enter hook returned true.
    #[inline]
    fn leave_jsx_fragment(&mut self, _fragment: &'a JSXFragment<'a>) {}

    /// A call expression, before the walk enters its callee and arguments.
    #[inline]
    fn call_expression(&mut self, _call: &'a CallExpression<'a>) {}
}

/// Walk a whole program.
pub(crate) fn walk_program<'a>(nodes: &mut impl LentNodes<'a>, program: &'a Program<'a>) {
    generated::walk_program(nodes, program);
}

/// Walk one expression.
pub(crate) fn walk_expression<'a>(nodes: &mut impl LentNodes<'a>, expression: &'a Expression<'a>) {
    generated::walk_expression(nodes, expression);
}

/// Walk one JSX expression container.
pub(crate) fn walk_jsx_expression_container<'a>(
    nodes: &mut impl LentNodes<'a>,
    container: &'a JSXExpressionContainer<'a>,
) {
    generated::walk_jsx_expression_container(nodes, container);
}
