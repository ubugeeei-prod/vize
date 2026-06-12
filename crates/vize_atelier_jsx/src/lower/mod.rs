//! Lowering OXC JSX nodes into Vize's shared template IR.
//!
//! The [`Lowerer`] walks OXC JSX/TSX nodes and produces
//! [`vize_relief::ast::RootNode`]s. Owned strings are copied out of the OXC
//! arena (Vize uses `CompactString`), and the tree structure is built in the
//! caller-supplied [`Bump`] arena, so the lowered IR does not borrow the OXC
//! allocator and outlives parsing.

mod attr;
mod child;
mod control_flow;
mod element;
mod expr;
mod name;
mod slot;
mod style;
mod text;

pub(crate) use style::{RawScopedStyle, ScopedStyleExpr};

use oxc_ast::ast::{JSXElement, JSXFragment};
use vize_carton::{Box, Bump, String};
use vize_relief::ast::{RootNode, TemplateChildNode};

use crate::diagnostics::JsxDiagnostic;
use crate::span::SpanMapper;

/// Lowers OXC JSX nodes into Vize IR against a single source text.
pub struct Lowerer<'a, 'm, 's> {
    bump: &'a Bump,
    mapper: &'m SpanMapper<'s>,
    diagnostics: std::vec::Vec<JsxDiagnostic>,
    /// `<style scoped>` blocks extracted from the render root currently being
    /// lowered, in source order. Drained by [`Self::take_scoped_styles`] once
    /// the root is built.
    pending_styles: std::vec::Vec<RawScopedStyle>,
}

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Build a lowerer that allocates IR in `bump` and maps spans via `mapper`.
    pub fn new(bump: &'a Bump, mapper: &'m SpanMapper<'s>) -> Self {
        Self {
            bump,
            mapper,
            diagnostics: std::vec::Vec::new(),
            pending_styles: std::vec::Vec::new(),
        }
    }

    /// Record a `<style scoped>` block extracted during child lowering.
    pub(crate) fn push_scoped_style(&mut self, style: RawScopedStyle) {
        self.pending_styles.push(style);
    }

    /// Drain the `<style scoped>` blocks accumulated while lowering the current
    /// render root, concatenating their CSS into one block (multiple `<style
    /// scoped>` elements in one component join, mirroring SFC's multi-`<style>`
    /// behavior) and flattening every template-literal interpolation expression
    /// (`${expr}`) across them, in source order. Returns `None` when no scoped
    /// style was present.
    pub(crate) fn take_scoped_styles(
        &mut self,
    ) -> Option<(String, std::vec::Vec<ScopedStyleExpr>)> {
        if self.pending_styles.is_empty() {
            return None;
        }
        let styles = std::mem::take(&mut self.pending_styles);
        let mut css = String::default();
        let mut exprs = std::vec::Vec::new();
        for (index, style) in styles.into_iter().enumerate() {
            if index > 0 {
                css.push('\n');
            }
            css.push_str(style.css.trim());
            exprs.extend(style.exprs);
        }
        Some((css, exprs))
    }

    /// Diagnostics accumulated so far.
    pub fn diagnostics(&self) -> &[JsxDiagnostic] {
        &self.diagnostics
    }

    /// Consume the lowerer and return its diagnostics.
    pub fn into_diagnostics(self) -> std::vec::Vec<JsxDiagnostic> {
        self.diagnostics
    }

    /// Record a diagnostic.
    pub fn report(&mut self, diagnostic: JsxDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Lower a JSX element as the single root of a render output.
    pub fn lower_element_root(&mut self, element: &JSXElement<'_>) -> RootNode<'a> {
        let mut root = RootNode::new(self.bump, self.mapper.slice(element.span));
        root.loc = self.mapper.location(element.span);
        let node = self.lower_element_node(element);
        root.children
            .push(TemplateChildNode::Element(Box::new_in(node, self.bump)));
        root
    }

    /// Lower a JSX fragment (`<>...</>`) as a render root whose children become
    /// the root children directly (no wrapper element).
    pub fn lower_fragment_root(&mut self, fragment: &JSXFragment<'_>) -> RootNode<'a> {
        let mut root = RootNode::new(self.bump, self.mapper.slice(fragment.span));
        root.loc = self.mapper.location(fragment.span);
        root.children = self.lower_children(&fragment.children);
        root
    }

    /// Shared accessor used by sibling lowering modules.
    pub(crate) fn bump(&self) -> &'a Bump {
        self.bump
    }

    /// Shared accessor used by sibling lowering modules.
    pub(crate) fn mapper(&self) -> &'m SpanMapper<'s> {
        self.mapper
    }
}
