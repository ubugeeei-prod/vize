//! Projection of the admitted native payload to the shared emitter IR.
//! The legacy template AST is neither an input nor a source of payloads.
//!
//! Node ids follow upstream as pinned by the published fixture corpus: slot
//! content before its component, component and outlet children before their
//! parent element, the element before its other dynamic descendants, and one
//! reserved id per control-flow block. Ids are names only; no runtime
//! behavior depends on them. Blocks, text roots and templates otherwise take
//! the retained lane's shapes, so admitted templates are byte-identical.

mod bindings;
mod component;
mod control;
mod element;
mod spans;
mod spread;
mod text;

use vize_atelier_core::{
    RootNode, SimpleExpressionNode, SourceLocation, codegen::document::EmitDocument,
};
use vize_carton::{Allocator, Box, FxHashMap, String, Vec, ensure_sufficient_stack};

use super::{AuthoredSpan, Content, Expr, NativeArtifact};
use crate::generate::spans::{TemplateSpans, VaporSourceSpans};
use crate::ir::{BlockIRNode, ChildRefIRNode, IREffect, NextRefIRNode, OperationNode, RootIRNode};

/// The shared emitter IR for `artifact`; with `spans`, also the template and
/// control-flow anchors a source map needs (Davinci P3-9). `None` when the
/// payload breaks a shape admission checked; the caller then keeps the
/// legacy lane instead of emitting a partial render.
pub(super) fn emit<'a>(
    mut artifact: NativeArtifact<'a>,
    allocator: &'a Allocator,
    source: &'a str,
    scope_id: Option<&str>,
    spans: bool,
) -> Option<(RootIRNode<'a>, Option<VaporSourceSpans>)> {
    let mut ir = RootIRNode {
        node: RootNode::new(allocator, ""),
        source,
        template: Default::default(),
        template_index_map: Default::default(),
        root_template_indexes: Vec::new_in(&allocator),
        component: Vec::new_in(&allocator),
        directive: Vec::new_in(&allocator),
        block: BlockIRNode::new(allocator),
        has_template_ref: false,
        has_deferred_v_show: false,
        templates: Vec::new_in(&allocator),
        element_template_map: Default::default(),
        standalone_text_elements: Default::default(),
    };
    // A node is materialized when it or a descendant in the same template is
    // updated, listened to, or anchors an insertion (control flow, slots).
    let mut dynamic = Vec::with_capacity_in(artifact.nodes.len(), &allocator);
    dynamic.resize(artifact.nodes.len(), false);
    for (index, node) in artifact.nodes.iter().enumerate().rev() {
        let is_dynamic = match node.content {
            Content::Text { dynamic, .. } => dynamic,
            Content::Element { .. } => {
                !node.bindings.is_empty()
                    || (node.children.iter()).any(|child| dynamic.get(*child) == Some(&true))
            }
            Content::If { .. }
            | Content::For(_)
            | Content::Component { .. }
            | Content::Outlet { .. } => true,
        };
        if let Some(slot) = dynamic.get_mut(index) {
            *slot = is_dynamic;
        }
    }
    ir.element_template_map.reserve(artifact.nodes.len());
    let roots = take(allocator, &mut artifact.roots);
    let mut emitter = Emitter {
        allocator,
        artifact,
        dynamic,
        ir: &mut ir,
        next_id: 0,
        scope_id,
        source: spans.then_some(source),
        template_spans: TemplateSpans::default(),
        units: FxHashMap::default(),
        else_units: FxHashMap::default(),
        tags: FxHashMap::default(),
        broken: core::cell::Cell::new(false),
    };
    let block = emitter.block(&roots);
    if emitter.broken.get() {
        return None;
    }
    let spans = spans.then(|| {
        VaporSourceSpans::native(
            std::mem::take(&mut emitter.template_spans),
            std::mem::take(&mut emitter.units),
            std::mem::take(&mut emitter.else_units),
            std::mem::take(&mut emitter.tags),
        )
    });
    ir.block = block;
    Some((ir, spans))
}

struct Emitter<'a, 'b> {
    allocator: &'a Allocator,
    artifact: NativeArtifact<'a>,
    dynamic: Vec<'a, bool>,
    ir: &'b mut RootIRNode<'a>,
    next_id: usize,
    scope_id: Option<&'b str>,
    /// The authored source, only for map-requesting compiles.
    source: Option<&'a str>,
    template_spans: TemplateSpans,
    /// Expression span start -> authored start of its control-flow or slot unit.
    units: FxHashMap<u32, u32>,
    /// Condition span start -> authored start of the following `v-else`.
    else_units: FxHashMap<u32, u32>,
    /// Component tag -> authored start of its tag name (the byte after `<`).
    tags: FxHashMap<String, u32>,
    /// Set when the payload breaks a shape admission checked; the render is
    /// then discarded (see [`emit`]).
    broken: core::cell::Cell<bool>,
}

impl<'a> Emitter<'a, '_> {
    /// One render block over `children`, in the retained lane's
    /// `transform_children` order: a pure text run becomes one text node,
    /// otherwise every child is instantiated or created in turn.
    fn block(&mut self, children: &[usize]) -> BlockIRNode<'a> {
        self.block_with(children, true)
    }

    /// A branch or loop body. The retained lane transforms a `<template>`
    /// carrier's children one by one, so its text parts are never combined.
    fn body(&mut self, children: &[usize]) -> BlockIRNode<'a> {
        self.block_with(children, false)
    }

    fn block_with(&mut self, children: &[usize], combine: bool) -> BlockIRNode<'a> {
        ensure_sufficient_stack(|| {
            let mut block = BlockIRNode::new(self.allocator);
            if combine && self.combined_text(children, &mut block) {
                return block;
            }
            for &child in children {
                let Some(node) = self.artifact.nodes.get(child) else {
                    self.invariant_broken();
                    continue;
                };
                match node.content {
                    Content::Element { .. } => self.element(child, &mut block),
                    Content::Text { .. } => self.text_roots(child, &mut block),
                    Content::If { .. } | Content::For(_) => {
                        let id = self.control(child, None, &mut block);
                        block.returns.push(id);
                    }
                    Content::Component { .. } => self.component(child, None, None, &mut block),
                    Content::Outlet { .. } => {
                        let id = self.id();
                        self.outlet(child, id, &mut block);
                        block.returns.push(id);
                    }
                }
            }
            block
        })
    }

    /// Record that the payload broke an admission-checked shape.
    fn invariant_broken(&self) {
        self.broken.set(true);
    }

    fn id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn register(&mut self, id: usize, template: &EmitDocument) {
        let index = self.ir.templates.len();
        self.ir.element_template_map.insert(id, index);
        self.ir
            .templates
            .push(self.allocator.alloc_str(template.as_str()));
        if self.source.is_some() && !template.links().is_empty() {
            self.template_spans.insert(index, template.links().to_vec());
        }
    }

    fn child(&mut self, parent_id: usize, offset: usize, block: &mut BlockIRNode<'a>) -> usize {
        let child_id = self.id();
        block
            .operation
            .push(OperationNode::ChildRef(ChildRefIRNode {
                child_id,
                parent_id,
                offset,
            }));
        child_id
    }

    /// A sibling reached `offset` positions after an already referenced one.
    fn next(&mut self, prev_id: usize, offset: usize, block: &mut BlockIRNode<'a>) -> usize {
        let child_id = self.id();
        block.operation.push(OperationNode::NextRef(NextRefIRNode {
            child_id,
            prev_id,
            offset,
        }));
        child_id
    }

    /// A generator expression node; a retained AST lets the shared resolver
    /// consume it without reparsing the text.
    fn expression(&self, value: Expr<'a>, is_static: bool) -> Box<'a, SimpleExpressionNode<'a>> {
        self.spanned(value, is_static, None)
    }

    /// [`Self::expression`] that keeps the payload's authored `span` for a
    /// map-requesting compile.
    fn spanned(
        &self,
        value: Expr<'a>,
        is_static: bool,
        span: Option<AuthoredSpan>,
    ) -> Box<'a, SimpleExpressionNode<'a>> {
        let loc = span
            .filter(|_| self.source.is_some())
            .map_or(SourceLocation::STUB, |(start, end)| {
                SourceLocation::new(start, end)
            });
        let mut node = SimpleExpressionNode::new(value.text, is_static, loc);
        node.js_ast = value.js;
        Box::new_in(node, &self.allocator)
    }

    fn values(
        &self,
        value: Expr<'a>,
        span: Option<AuthoredSpan>,
    ) -> Vec<'a, Box<'a, SimpleExpressionNode<'a>>> {
        let mut values = Vec::new_in(&self.allocator);
        values.push(self.spanned(value, false, span));
        values
    }

    fn effect(&mut self, op: OperationNode<'a>, block: &mut BlockIRNode<'a>) {
        let mut operations = Vec::new_in(&self.allocator);
        operations.push(op);
        block.effect.push(IREffect { operations });
    }
}

/// Move a payload list out of the artifact; each node is emitted once.
fn take<'a, T>(allocator: &'a Allocator, list: &mut Vec<'a, T>) -> Vec<'a, T> {
    std::mem::replace(list, Vec::new_in(&allocator))
}

fn escaped(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            ch => output.push(ch),
        }
    }
    output
}
