//! Projection of the admitted native payload to the shared emitter IR.
//! The legacy template AST is neither an input nor a source of payloads.

mod bindings;
mod control;
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
    // updated, listened to, or anchors a control-flow insertion.
    let mut dynamic = std::vec![false; artifact.nodes.len()];
    for (index, node) in artifact.nodes.iter().enumerate().rev() {
        dynamic[index] = !node.bindings.is_empty()
            || matches!(
                node.content,
                Content::Text { dynamic: true, .. } | Content::If { .. } | Content::For(_)
            )
            || node.children.iter().any(|child| dynamic[*child]);
    }
    let root = artifact.root;
    let mut emitter = Emitter {
        allocator,
        artifact,
        dynamic,
        ir: &mut ir,
        next_id: 0,
        scope_id,
    };
    let block = emitter.block(root);
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
    /// One render block: an element instantiated from its own template, or a
    /// control-flow operation at the template root.
    fn block(&mut self, root: usize) -> BlockIRNode<'a> {
        ensure_sufficient_stack(|| {
            let mut block = BlockIRNode::new(self.allocator);
            let id = if matches!(self.artifact.nodes[root].content, Content::Element { .. }) {
                let id = self.id();
                let mut template = String::default();
                self.element(root, Some(id), &mut template, &mut block);
                // Nested blocks registered their templates first.
                self.ir
                    .element_template_map
                    .insert(id, self.ir.templates.len());
                self.ir.templates.push(self.allocator.alloc_str(&template));
                id
            } else {
                self.control(root, None, &mut block)
            };
            block.returns.push(id);
            block
        })
    }

    fn element(
        &mut self,
        index: usize,
        id: Option<usize>,
        template: &mut String,
        block: &mut BlockIRNode<'a>,
    ) {
        ensure_sufficient_stack(|| self.element_inner(index, id, template, block));
    }

    fn element_inner(
        &mut self,
        index: usize,
        id: Option<usize>,
        template: &mut String,
        block: &mut BlockIRNode<'a>,
    ) {
        let Content::Element {
            tag,
            ref attributes,
        } = self.artifact.nodes[index].content
        else {
            unreachable!("element payload checked by the caller")
        };
        template.push('<');
        template.push_str(tag);
        for (name, value) in attributes {
            template.push(' ');
            template.push_str(name);
            if let Some(value) = value {
                template.push_str("=\"");
                escape(template, value);
                template.push('"');
            }
        }
        if let Some(scope_id) = self.scope_id {
            template.push(' ');
            template.push_str(scope_id);
        }
        template.push('>');
        for binding in 0..self.artifact.nodes[index].bindings.len() {
            let element = id.expect("binding target is materialized");
            self.binding(index, binding, element, block);
        }
        self.children(index, id, template, block);
        if !vize_carton::is_void_tag(tag) {
            template.push_str("</");
            template.push_str(tag);
            template.push('>');
        }
    }

    fn children(
        &mut self,
        index: usize,
        parent: Option<usize>,
        template: &mut String,
        block: &mut BlockIRNode<'a>,
    ) {
        let only_child = self.artifact.nodes[index].children.len() == 1;
        let mut cursor = 0;
        let mut offset = 0;
        while cursor < self.artifact.nodes[index].children.len() {
            let child = self.artifact.nodes[index].children[cursor];
            match self.artifact.nodes[child].content {
                Content::Text { .. } => {
                    cursor = self.text_run(index, cursor, parent, offset, template, block);
                }
                Content::Element { .. } => {
                    let id = self.dynamic[child].then(|| {
                        self.child(
                            parent.expect("dynamic ancestry is materialized"),
                            offset,
                            block,
                        )
                    });
                    self.element(child, id, template, block);
                    cursor += 1;
                }
                Content::If { .. } | Content::For(_) => {
                    // The placeholder is the authored insertion position; the
                    // control block inserts before it.
                    template.push_str("<!---->");
                    let parent = parent.expect("control-flow parent is materialized");
                    let anchor = self.child(parent, offset, block);
                    self.control(child, Some((parent, anchor, only_child)), block);
                    cursor += 1;
                }
            }
            offset += 1;
        }
    }

    fn id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
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
