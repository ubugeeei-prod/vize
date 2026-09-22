//! Zero-copy, rule-facing markup IR shared by Vue-template and JSX/TSX rules.
//!
//! # Why
//!
//! Patina rules historically receive concrete `vize_relief` template nodes
//! (`ElementNode`, `DirectiveNode`, `ForNode`, `IfNode`, …). Those types only
//! exist for Vue templates, so a rule written against them cannot run over
//! JSX/TSX, where bindings are OXC `JSXAttribute`s, events are `onClick`-style
//! props, and `v-for` / `v-if` show up structurally as `items.map(...)` and
//! `cond && <x/>` rather than as directives.
//!
//! This module lifts the rule API onto a small, typed, **borrow-based** facade
//! projected from three backends without materializing a synthetic AST:
//!
//! - **S2 (Disegno)** — the Davinci semantic IR ([`S2Markup`]): SFC templates
//!   through the S1→S2 lowering, JSX/TSX through the P2-16 S2 projection. This
//!   is the backend the lint lanes converge on (Davinci P4-7).
//! - **Relief** — a `vize_relief` template root (a raw template parse, or a
//!   JSX root lowered to Relief).
//! - **OXC** — a JSX/TSX program read straight from the OXC AST.
//!
//! Every wrapper is `Copy` and holds a shared reference into the backing arena
//! (or, for OXC, a raw pointer into the program kept alive for the lint pass —
//! see the `jsx_*_ref` SAFETY notes). Names and values are `&str` slices that
//! borrow the original source; nothing allocates unless a rule explicitly asks
//! for normalized owned data (e.g. [`MarkupElement::direct_text_content`]).
//!
//! # One contract: structured control flow
//!
//! The facade presents control flow the way S2 does — as **scopes, not
//! directive attributes**. A `v-if` / `v-else-if` / `v-else` sibling chain
//! surfaces as one [`MarkupConditional`], a `v-for` as one [`MarkupList`], and
//! an unslotted `<template v-if>` / `<template v-for>` wrapper is unwrapped
//! into its scope. The structural directives themselves are consumed: they are
//! never reported as bindings or directives. [`MarkupElement::walk_children`]
//! stays the *authored* child list — a template's `v-if` / `v-for` carrier is
//! the child element, a JSX conditional or list expression one scope node —
//! so rule bodies that read content see what was written. The Relief backend synthesizes
//! the same scopes from a raw template parse ([`relief_scopes`]) so every
//! backend answers identically; the `davinci-differential` lane
//! ([`differential`]) proves it by comparing full hook traces exactly.
//!
//! # Shape
//!
//! - [`MarkupDocument`] — document-level entry point, optionally carrying a
//!   [`Croquis`] for semantic / type-aware rules.
//! - [`MarkupElement`] — element / component / fragment / template / slot node.
//! - [`MarkupAttribute`] — a *written* static attribute.
//! - [`MarkupDirective`] — a Vue directive **or** a directive-like JSX
//!   attribute (so `walk_directives` is meaningful on JSX too).
//! - [`MarkupBinding`] — the normalized binding view ([`MarkupBindingKind`]).
//! - [`MarkupConditional`] / [`MarkupList`] — conditional and list scopes.
//! - [`MarkupNode`] — a child node (element, text, interpolation, scope,
//!   comment).
//!
//! # Driving rules
//!
//! Implement [`MarkupRule`] (default-empty `enter_*` hooks) and run it with
//! [`MarkupDocumentVisitor`], which projects the hooks from any backend. The
//! visitor threads a [`MarkupContext`] wrapping the existing
//! [`LintContext`](crate::context::LintContext), so rules keep using the same
//! diagnostic/fix APIs and all source ranges map back to the original syntax.

use crate::ir::{ByteRange, TemplateSyntax};
use oxc_ast::ast::Program;
use oxc_span::Span;
use vize_croquis::Croquis;
use vize_relief::{RootNode, SourceLocation};

mod attribute;
mod binding;
mod context;
#[cfg(any(test, feature = "davinci-differential"))]
pub mod differential;
mod directive;
mod dispatch;
mod element;
mod exact;
mod hooks;
mod jsx_child;
mod jsx_names;
mod jsx_roots;
mod node;
mod relief_children;
mod relief_scopes;
mod rule;
mod s2;
mod scope;
mod source_ranges;
#[cfg(test)]
mod tests;
mod visitor;

pub use attribute::MarkupAttribute;
pub use binding::{MarkupBinding, MarkupBindingKind};
pub use context::MarkupContext;
pub use directive::MarkupDirective;
pub use dispatch::{MarkupRuleLayout, MarkupRuleSet, MarkupRules};
pub use element::MarkupElement;
pub use hooks::MarkupHooks;
pub use node::{MarkupNode, MarkupText};
pub use rule::MarkupRule;
pub use s2::{S2Markup, S2Template};
pub use scope::{MarkupConditional, MarkupList};
pub use visitor::MarkupDocumentVisitor;

/// High-level classification for a markup element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupElementKind {
    /// Plain HTML element.
    Element,
    /// Framework component.
    Component,
    /// Slot outlet.
    Slot,
    /// Template wrapper.
    Template,
}

#[derive(Clone, Copy)]
enum MarkupDocumentInner<'a> {
    Relief(&'a RootNode<'a>),
    Jsx {
        program: &'a Program<'a>,
        offset: u32,
    },
    S2(&'a S2Markup<'a>),
}

/// Document-level markup view.
///
/// Borrows an S2 artifact, a `vize_relief` template root, or an OXC JSX/TSX
/// program, and optionally a [`Croquis`] for semantic / type-aware rules.
/// `Copy` so it can be passed by value into the [`MarkupDocumentVisitor`].
#[derive(Clone, Copy)]
pub struct MarkupDocument<'a> {
    inner: MarkupDocumentInner<'a>,
    syntax: TemplateSyntax,
    analysis: Option<&'a Croquis>,
}

impl<'a> MarkupDocument<'a> {
    /// Create a markup document from a parsed template root.
    pub const fn new(root: &'a RootNode<'a>, syntax: TemplateSyntax) -> Self {
        Self {
            inner: MarkupDocumentInner::Relief(root),
            syntax,
            analysis: None,
        }
    }

    /// Create a markup document from a parsed JSX/TSX program.
    pub const fn from_jsx(program: &'a Program<'a>, syntax: TemplateSyntax, offset: u32) -> Self {
        Self {
            inner: MarkupDocumentInner::Jsx { program, offset },
            syntax,
            analysis: None,
        }
    }

    /// Create a markup document viewing an S2 (Disegno) artifact.
    ///
    /// The view is zero-copy: elements, bindings, text, and scopes borrow the
    /// S2 op tree and its lowering-published side tables directly.
    pub const fn from_s2(markup: &'a S2Markup<'a>, syntax: TemplateSyntax) -> Self {
        Self {
            inner: MarkupDocumentInner::S2(markup),
            syntax,
            analysis: None,
        }
    }

    /// Attach optional [`Croquis`] semantic analysis to this document.
    ///
    /// Carried through to [`MarkupContext::analysis`] so type-aware and
    /// semantic rules can reach the same analysis the rest of the pipeline saw,
    /// without re-deriving it.
    pub const fn with_analysis(mut self, analysis: &'a Croquis) -> Self {
        self.analysis = Some(analysis);
        self
    }

    /// Semantic analysis attached to this document, if any.
    pub const fn analysis(&self) -> Option<&'a Croquis> {
        self.analysis
    }

    /// Whether this document was projected from a Vue template rather than
    /// from JSX/TSX. Lets rules apply template-only semantics where they make
    /// sense.
    pub const fn is_template(&self) -> bool {
        match self.inner {
            MarkupDocumentInner::Relief(_) => true,
            MarkupDocumentInner::Jsx { .. } => false,
            MarkupDocumentInner::S2(markup) => markup.is_template(),
        }
    }

    /// Whether this document was projected from JSX/TSX.
    pub const fn is_jsx(&self) -> bool {
        !self.is_template()
    }

    /// Whether this document views an S2 artifact.
    pub const fn is_s2(&self) -> bool {
        matches!(self.inner, MarkupDocumentInner::S2(_))
    }

    /// Template syntax used by this document.
    pub const fn syntax(&self) -> TemplateSyntax {
        self.syntax
    }

    /// Walk all concrete elements in tree order.
    pub fn walk_elements(&self, visitor: &mut impl FnMut(MarkupElement<'a>)) {
        self.walk_tree(visitor, &mut |_| {});
    }

    /// Walk the full element tree, calling enter/exit callbacks.
    pub fn walk_tree(
        &self,
        enter: &mut impl FnMut(MarkupElement<'a>),
        exit: &mut impl FnMut(MarkupElement<'a>),
    ) {
        match self.inner {
            MarkupDocumentInner::Relief(root) => {
                relief_children::walk_relief_children(&root.children, enter, exit);
            }
            MarkupDocumentInner::Jsx { program, offset } => {
                jsx_roots::walk_jsx_program(program, offset, enter, exit)
            }
            MarkupDocumentInner::S2(markup) => s2::walk::walk_tree(markup, enter, exit),
        }
    }

    /// Drive a [`MarkupRule`] over this document through a [`MarkupContext`].
    ///
    /// This is the zero-copy projection visitor: it walks the backing tree in
    /// source order and fires the rule's `enter_*` hooks for elements,
    /// attributes, directives, bindings, conditional / list scopes, text, and
    /// interpolation/expression nodes — without ever allocating a synthetic
    /// template AST.
    pub fn visit_with<R: MarkupRule + ?Sized>(&self, rule: &R, ctx: &mut MarkupContext<'_, 'a>) {
        MarkupDocumentVisitor::new(&dispatch::One::new(rule), ctx).run(self);
    }

    /// Drive a fused rule set over this document in one walk: at every node,
    /// each subscribed rule's hook fires in registration order (the template
    /// visitor's dispatch shape), so the document is walked once, not once
    /// per rule.
    pub fn visit_rules(&self, rules: &MarkupRuleSet<'_>, ctx: &mut MarkupContext<'_, 'a>) {
        MarkupDocumentVisitor::new(rules, ctx).run(self);
    }
}

#[inline]
fn span_to_range(span: Span, offset: u32) -> ByteRange {
    ByteRange::new(offset + span.start, offset + span.end)
}

#[inline]
fn loc_to_range(loc: &SourceLocation) -> ByteRange {
    ByteRange::new(loc.span.start, loc.span.end)
}

#[inline]
fn s2_range(span: vize_s0::Span) -> ByteRange {
    ByteRange::new(span.start, span.end)
}
