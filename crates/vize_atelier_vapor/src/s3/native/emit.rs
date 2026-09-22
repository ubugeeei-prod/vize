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
mod spread;
mod text;

use oxc_allocator::StringBuilder;
use vize_atelier_core::{RootNode, SimpleExpressionNode, SourceLocation};
use vize_carton::{Allocator, Box, Vec, ensure_sufficient_stack};

use super::{Content, Expr, NativeArtifact};
use crate::ir::{BlockIRNode, ChildRefIRNode, IREffect, NextRefIRNode, OperationNode, RootIRNode};

pub(super) fn emit<'a>(
    mut artifact: NativeArtifact<'a>,
    allocator: &'a Allocator,
    source: &'a str,
    scope_id: Option<&str>,
) -> RootIRNode<'a> {
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
        dynamic[index] = match node.content {
            Content::Text { dynamic, .. } => dynamic,
            Content::Element { .. } => {
                !node.bindings.is_empty() || node.children.iter().any(|child| dynamic[*child])
            }
            Content::If { .. }
            | Content::For(_)
            | Content::Component { .. }
            | Content::Outlet { .. } => true,
        };
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
    };
    let block = emitter.block(&roots);
    ir.block = block;
    ir
}

struct Emitter<'a, 'b> {
    allocator: &'a Allocator,
    artifact: NativeArtifact<'a>,
    dynamic: Vec<'a, bool>,
    ir: &'b mut RootIRNode<'a>,
    next_id: usize,
    scope_id: Option<&'b str>,
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
                match self.artifact.nodes[child].content {
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

    fn id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Templates are built in the output arena, so registering copies nothing.
    fn register(&mut self, id: usize, template: &'a str) {
        self.ir
            .element_template_map
            .insert(id, self.ir.templates.len());
        self.ir.templates.push(template);
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
        let mut node = SimpleExpressionNode::new(value.text, is_static, SourceLocation::STUB);
        node.js_ast = value.js;
        Box::new_in(node, &self.allocator)
    }

    fn values(&self, value: Expr<'a>) -> Vec<'a, Box<'a, SimpleExpressionNode<'a>>> {
        let mut values = Vec::new_in(&self.allocator);
        values.push(self.expression(value, false));
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

/// HTML-escape `value` into `output`, copying unescaped runs whole.
fn escape(output: &mut StringBuilder<'_>, value: &str) {
    let mut start = 0;
    for (index, byte) in value.bytes().enumerate() {
        let entity = match byte {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&quot;",
            b'\'' => "&#39;",
            _ => continue,
        };
        output.push_str(&value[start..index]);
        output.push_str(entity);
        start = index + 1;
    }
    output.push_str(&value[start..]);
}
