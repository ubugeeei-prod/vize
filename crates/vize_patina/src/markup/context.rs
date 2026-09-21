//! [`MarkupContext`]: the context handed to [`MarkupRule`](super::MarkupRule) callbacks.

use super::MarkupDocument;
use super::element::MarkupElement;
use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use vize_croquis::Croquis;
use vize_s0::SmallVec;

/// Context handed to [`MarkupRule`](super::MarkupRule) callbacks.
///
/// Thin wrapper over the existing [`LintContext`] (so rules keep the full
/// diagnostic / fix / semantic API and `ByteRange`-based reporting via
/// [`LintContext::error_at`] and friends) plus the document-level metadata a
/// markup rule typically needs: source syntax, whether the input is a template
/// or JSX, and the optional [`Croquis`].
pub struct MarkupContext<'ctx, 'a> {
    pub(super) lint: &'ctx mut LintContext<'a>,
    syntax: TemplateSyntax,
    is_template: bool,
    analysis: Option<&'a Croquis>,
    element_stack: SmallVec<[MarkupElement<'a>; 32]>,
    jsx_attribute_value_depth: u32,
}

impl<'ctx, 'a> MarkupContext<'ctx, 'a> {
    /// Build a markup context from a [`LintContext`] and the document being
    /// linted.
    ///
    /// Semantic analysis is taken from the document's [`MarkupDocument::analysis`]
    /// when present; callers that only have analysis on the [`LintContext`]
    /// should attach it to the document with [`MarkupDocument::with_analysis`].
    pub fn new(lint: &'ctx mut LintContext<'a>, document: &MarkupDocument<'a>) -> Self {
        Self {
            syntax: document.syntax(),
            is_template: document.is_template(),
            analysis: document.analysis(),
            element_stack: SmallVec::new(),
            jsx_attribute_value_depth: 0,
            lint,
        }
    }

    /// Mutable access to the underlying [`LintContext`], for reporting
    /// diagnostics and reaching the full semantic API.
    #[inline]
    pub fn lint(&mut self) -> &mut LintContext<'a> {
        &mut *self.lint
    }

    /// The document's template syntax.
    #[inline]
    pub fn syntax(&self) -> TemplateSyntax {
        self.syntax
    }

    /// Whether the document is a Vue template (rather than JSX/TSX). Useful for
    /// directive-only semantics that have no JSX analogue.
    #[inline]
    pub fn is_template(&self) -> bool {
        self.is_template
    }

    /// Whether the document is JSX/TSX.
    #[inline]
    pub fn is_jsx(&self) -> bool {
        !self.is_template
    }

    /// The optional [`Croquis`] semantic analysis for this document.
    #[inline]
    pub fn analysis(&self) -> Option<&'a Croquis> {
        self.analysis
    }

    /// Current element being visited.
    ///
    /// The current element is pushed before [`MarkupRule::enter_element`] and
    /// popped after [`MarkupRule::exit_element`], matching the legacy template
    /// visitor's [`LintContext`] element-stack timing.
    #[inline]
    pub fn current_element(&self) -> Option<MarkupElement<'a>> {
        self.element_stack.last().copied()
    }

    /// Parent element of the current markup element, if any.
    #[inline]
    pub fn parent_element(&self) -> Option<MarkupElement<'a>> {
        self.element_stack
            .get(self.element_stack.len().checked_sub(2)?)
            .copied()
    }

    /// Ancestor elements of the current markup element, root first.
    #[inline]
    pub fn ancestor_elements(&self) -> impl DoubleEndedIterator<Item = MarkupElement<'a>> + '_ {
        let ancestor_len = self.element_stack.len().saturating_sub(1);
        self.element_stack[..ancestor_len].iter().copied()
    }

    /// Check whether any ancestor of the current element matches `predicate`.
    #[inline]
    pub fn has_ancestor(&self, predicate: impl FnMut(MarkupElement<'a>) -> bool) -> bool {
        self.ancestor_elements().any(predicate)
    }

    /// Whether the current JSX element was reached through an attribute value.
    ///
    /// The legacy JSX fallback only lowered render roots and their children. It
    /// did not lint JSX passed as props such as `<Comp render={<h1 />} />`.
    /// Rules that are migrating from the fallback can use this to preserve that
    /// visible boundary while the facade still exposes attribute-value roots for
    /// rules that intentionally inspect them.
    #[inline]
    pub const fn is_jsx_attribute_value(&self) -> bool {
        self.jsx_attribute_value_depth > 0
    }

    #[inline]
    pub(super) fn push_element(&mut self, element: MarkupElement<'a>) {
        self.element_stack.push(element);
    }

    #[inline]
    pub(super) fn pop_element(&mut self) -> Option<MarkupElement<'a>> {
        self.element_stack.pop()
    }

    #[inline]
    pub(super) fn push_jsx_attribute_value(&mut self) {
        self.jsx_attribute_value_depth += 1;
    }

    #[inline]
    pub(super) fn pop_jsx_attribute_value(&mut self) {
        self.jsx_attribute_value_depth = self.jsx_attribute_value_depth.saturating_sub(1);
    }
}
