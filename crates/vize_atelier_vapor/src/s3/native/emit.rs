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
mod text;

use vize_atelier_core::{RootNode, SimpleExpressionNode, SourceLocation};
use vize_carton::{Allocator, Box, String, Vec, ensure_sufficient_stack};

use super::{Content, Expr, NativeArtifact};
use crate::ir::{BlockIRNode, ChildRefIRNode, IREffect, OperationNode, RootIRNode};

pub(super) fn emit<'a>(
    artifact: NativeArtifact<'a>,
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
    let mut dynamic = std::vec![false; artifact.nodes.len()];
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
    let roots = artifact.roots.clone();
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
    dynamic: std::vec::Vec<bool>,
    ir: &'b mut RootIRNode<'a>,
    next_id: usize,
    scope_id: Option<&'b str>,
}

impl<'a> Emitter<'a, '_> {
    /// One render block over `children`, in the retained lane's
    /// `transform_children` order: a pure text run becomes one text node,
    /// otherwise every child is instantiated or created in turn.
    fn block(&mut self, children: &[usize]) -> BlockIRNode<'a> {
        ensure_sufficient_stack(|| {
            let mut block = BlockIRNode::new(self.allocator);
            if self.combined_text(children, &mut block) {
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

    fn register(&mut self, id: usize, template: &str) {
        self.ir
            .element_template_map
            .insert(id, self.ir.templates.len());
        self.ir.templates.push(self.allocator.alloc_str(template));
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

fn escape(output: &mut String, value: &str) {
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
}
