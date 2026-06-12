//! Lowering OXC JSX nodes into Vize's shared template IR.
//!
//! The [`Lowerer`] walks OXC JSX/TSX nodes and produces
//! [`vize_relief::ast::RootNode`]s. Owned strings are copied out of the OXC
//! arena (Vize uses `CompactString`), and the tree structure is built in the
//! caller-supplied [`Bump`] arena, so the lowered IR does not borrow the OXC
//! allocator and outlives parsing.

mod attr;
mod child;
mod element;
mod expr;
mod name;
mod slot;
mod text;

use oxc_ast::ast::{JSXElement, JSXFragment};
use vize_carton::{Box, Bump};
use vize_relief::ast::{RootNode, TemplateChildNode};

use crate::diagnostics::JsxDiagnostic;
use crate::span::SpanMapper;

/// Lowers OXC JSX nodes into Vize IR against a single source text.
pub struct Lowerer<'a, 'm, 's> {
    bump: &'a Bump,
    mapper: &'m SpanMapper<'s>,
    diagnostics: std::vec::Vec<JsxDiagnostic>,
}

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Build a lowerer that allocates IR in `bump` and maps spans via `mapper`.
    pub fn new(bump: &'a Bump, mapper: &'m SpanMapper<'s>) -> Self {
        Self {
            bump,
            mapper,
            diagnostics: std::vec::Vec::new(),
        }
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
