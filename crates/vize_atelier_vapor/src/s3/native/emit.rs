//! Projection of the admitted native payload to the shared emitter IR.
//! The legacy template AST is neither an input nor a source of payloads.

use vize_atelier_core::{RootNode, SimpleExpressionNode, SourceLocation};
use vize_carton::{Allocator, Box, String, Vec, ensure_sufficient_stack};

use super::{Content, NativeArtifact};
use crate::ir::{
    BlockIRNode, ChildRefIRNode, EventModifiers, IREffect, IRProp, OperationNode, RootIRNode,
    SetEventIRNode, SetPropIRNode, SetTextIRNode,
};

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
    let mut dynamic = std::vec![false; artifact.nodes.len()];
    for (index, node) in artifact.nodes.iter().enumerate().rev() {
        dynamic[index] = !node.bindings.is_empty()
            || matches!(node.content, Content::Text { dynamic: true, .. })
            || node.children.iter().any(|child| dynamic[*child]);
    }
    let mut emitter = Emitter {
        allocator,
        artifact,
        dynamic,
        ir: &mut ir,
        next_id: 0,
        scope_id,
    };
    let mut template = String::default();
    emitter.node(emitter.artifact.root, None, 0, &mut template);
    emitter.ir.templates.push(allocator.alloc_str(&template));
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
    fn node(&mut self, index: usize, parent: Option<usize>, offset: usize, template: &mut String) {
        ensure_sufficient_stack(|| self.node_inner(index, parent, offset, template));
    }

    fn node_inner(
        &mut self,
        index: usize,
        parent: Option<usize>,
        offset: usize,
        template: &mut String,
    ) {
        let id = if self.artifact.root == index {
            let id = self.next_id;
            self.next_id += 1;
            self.ir.block.returns.push(id);
            self.ir
                .element_template_map
                .insert(id, self.ir.templates.len());
            Some(id)
        } else if self.dynamic[index] {
            Some(self.child(parent.expect("dynamic ancestry is materialized"), offset))
        } else {
            None
        };
        let Content::Element {
            tag,
            ref attributes,
        } = self.artifact.nodes[index].content
        else {
            unreachable!("text handled above")
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
        for binding_index in 0..self.artifact.nodes[index].bindings.len() {
            let binding = &self.artifact.nodes[index].bindings[binding_index];
            let element = id.expect("binding target is materialized");
            let key = self.expression(binding.name, true);
            let op = if binding.event {
                let modifiers = EventModifiers::from_names(
                    self.allocator,
                    Some(binding.name),
                    binding.modifiers.iter().copied(),
                );
                let name = modifiers.event_name(binding.name);
                let delegate = modifiers.can_delegate(name);
                let key = self.expression(name, true);
                OperationNode::SetEvent(SetEventIRNode {
                    element,
                    key,
                    value: Some(self.expression(binding.value, false)),
                    modifiers,
                    delegate,
                    effect: false,
                })
            } else {
                OperationNode::SetProp(SetPropIRNode {
                    element,
                    tag,
                    camel: false,
                    prop_modifier: false,
                    prop: IRProp {
                        key,
                        values: self.values(binding.value),
                        is_component: false,
                    },
                })
            };
            if binding.event {
                self.ir.block.operation.push(op);
            } else {
                self.effect(op);
            }
        }
        self.children(index, id, template);
        if !vize_carton::is_void_tag(tag) {
            template.push_str("</");
            template.push_str(tag);
            template.push('>');
        }
    }

    fn children(&mut self, index: usize, parent: Option<usize>, template: &mut String) {
        let mut cursor = 0;
        let mut offset = 0;
        while cursor < self.artifact.nodes[index].children.len() {
            let child = self.artifact.nodes[index].children[cursor];
            if matches!(self.artifact.nodes[child].content, Content::Text { .. }) {
                let start = cursor;
                let mut dynamic = false;
                while cursor < self.artifact.nodes[index].children.len() {
                    let child = self.artifact.nodes[index].children[cursor];
                    let Content::Text { dynamic: part, .. } = self.artifact.nodes[child].content
                    else {
                        break;
                    };
                    dynamic |= part;
                    cursor += 1;
                }
                // HTML parsing coalesces adjacent text. One S3 text run must
                // own one DOM address, even when it contains many expressions.
                let mut values = Vec::new_in(&self.allocator);
                for position in start..cursor {
                    let child = self.artifact.nodes[index].children[position];
                    let Content::Text { ref parts, .. } = self.artifact.nodes[child].content else {
                        unreachable!("text run checked above")
                    };
                    for part in parts {
                        if dynamic {
                            values.push(self.expression(part.value, !part.dynamic));
                        }
                        if part.dynamic {
                            template.push(' ');
                        } else {
                            escape(template, part.value);
                        }
                    }
                }
                if dynamic {
                    let parent = parent.expect("dynamic ancestry is materialized");
                    let element = if offset == 0 {
                        parent
                    } else {
                        let id = self.child(parent, offset);
                        self.ir.standalone_text_elements.insert(id);
                        id
                    };
                    self.effect(OperationNode::SetText(SetTextIRNode {
                        element,
                        is_element: false,
                        values,
                    }));
                }
            } else {
                self.node(child, parent, offset, template);
                cursor += 1;
            }
            offset += 1;
        }
    }

    fn child(&mut self, parent_id: usize, offset: usize) -> usize {
        let child_id = self.next_id;
        self.next_id += 1;
        self.ir
            .block
            .operation
            .push(OperationNode::ChildRef(ChildRefIRNode {
                child_id,
                parent_id,
                offset,
            }));
        child_id
    }

    fn expression(&self, value: &'a str, is_static: bool) -> Box<'a, SimpleExpressionNode<'a>> {
        Box::new_in(
            SimpleExpressionNode::new(value, is_static, SourceLocation::STUB),
            &self.allocator,
        )
    }

    fn values(&self, value: &'a str) -> Vec<'a, Box<'a, SimpleExpressionNode<'a>>> {
        let mut values = Vec::new_in(&self.allocator);
        values.push(self.expression(value, false));
        values
    }

    fn effect(&mut self, op: OperationNode<'a>) {
        let mut operations = Vec::new_in(&self.allocator);
        operations.push(op);
        self.ir.block.effect.push(IREffect { operations });
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
