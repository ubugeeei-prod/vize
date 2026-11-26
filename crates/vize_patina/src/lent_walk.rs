//! oxc walks that lend each node for the borrow of the walked root.
//!
//! oxc's `Visit<'a>` passes every node to its callback as `&T<'a>`, borrowed
//! for that callback only, even when the walk started from a root borrowed for
//! `'a`. The markup facade keeps the JSX elements it finds and the
//! component-options rules keep the option objects they find for the whole
//! borrow of the program, so they need the nodes at `'a`.
//!
//! This module restores that lifetime once. Its visitor, `Lending`, is
//! private, and the only way to drive it is an entry point below whose root
//! is borrowed for `'a`. oxc's walk reaches each node from the root through
//! struct fields, `oxc_allocator::Vec<'a, _>` items and `oxc_allocator::Box<'a, _>`
//! targets, so every node it lends lies inside the root's borrow and stays
//! shared and alive for `'a`.

use oxc_ast::ast::{
    CallExpression, Expression, JSXElement, JSXExpressionContainer, JSXFragment, Program,
};
use oxc_ast_visit::{Visit, walk};

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
    walk::walk_program(&mut Lending { nodes }, program);
}

/// Walk one expression.
pub(crate) fn walk_expression<'a>(nodes: &mut impl LentNodes<'a>, expression: &'a Expression<'a>) {
    walk::walk_expression(&mut Lending { nodes }, expression);
}

/// Walk one JSX expression container.
pub(crate) fn walk_jsx_expression_container<'a>(
    nodes: &mut impl LentNodes<'a>,
    container: &'a JSXExpressionContainer<'a>,
) {
    walk::walk_jsx_expression_container(&mut Lending { nodes }, container);
}

/// The visitor behind the entry points. No other code can build one, so a
/// `Lending` only ever walks a root borrowed for `'a`.
struct Lending<'n, N> {
    nodes: &'n mut N,
}

impl<'a, N: LentNodes<'a>> Visit<'a> for Lending<'_, N> {
    #[inline]
    fn visit_jsx_element(&mut self, it: &JSXElement<'a>) {
        // SAFETY: `it` is a node of the root an entry point borrowed for `'a`
        // (module comment), so it stays shared and alive for `'a`.
        // A pointer cast changes only the lifetime; provenance is unchanged.
        let element: &'a JSXElement<'a> = unsafe { &*std::ptr::from_ref(it) };
        if self.nodes.enter_jsx_element(element) {
            walk::walk_jsx_element(self, it);
            self.nodes.leave_jsx_element(element);
        }
    }

    #[inline]
    fn visit_jsx_fragment(&mut self, it: &JSXFragment<'a>) {
        // SAFETY: as in `visit_jsx_element`.
        let fragment: &'a JSXFragment<'a> = unsafe { &*std::ptr::from_ref(it) };
        if self.nodes.enter_jsx_fragment(fragment) {
            walk::walk_jsx_fragment(self, it);
            self.nodes.leave_jsx_fragment(fragment);
        }
    }

    #[inline]
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        // SAFETY: as in `visit_jsx_element`.
        let call: &'a CallExpression<'a> = unsafe { &*std::ptr::from_ref(it) };
        self.nodes.call_expression(call);
        walk::walk_call_expression(self, it);
    }
}
