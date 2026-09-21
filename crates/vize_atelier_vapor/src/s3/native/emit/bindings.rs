//! Element bindings in authored order: props and content directives are
//! render effects; listeners and `v-show` are one-time operations.

use vize_atelier_core::{DirectiveNode, ExpressionNode, SourceLocation};
use vize_carton::{Box, Vec};

use super::super::{BindingKind, Content, Expr};
use super::Emitter;
use crate::ir::{
    BlockIRNode, DirectiveIRNode, EventModifiers, IRProp, OperationNode, SetEventIRNode,
    SetHtmlIRNode, SetPropIRNode, SetTextIRNode,
};

impl<'a> Emitter<'a, '_> {
    pub(super) fn binding(
        &mut self,
        index: usize,
        binding: usize,
        element: usize,
        block: &mut BlockIRNode<'a>,
    ) {
        let Content::Element {
            tag,
            ref attributes,
        } = self.artifact.nodes[index].content
        else {
            unreachable!("bindings attach to elements")
        };
        let input_type = attributes
            .iter()
            .find_map(|(name, value)| (*name == "type").then_some(*value).flatten())
            .unwrap_or("");
        let binding = &self.artifact.nodes[index].bindings[binding];
        match binding.kind {
            BindingKind::Event => {
                let modifiers = EventModifiers::from_names(
                    self.allocator,
                    Some(binding.name),
                    binding.modifiers.iter().copied(),
                );
                let name = modifiers.event_name(binding.name);
                let delegate = modifiers.can_delegate(name);
                block
                    .operation
                    .push(OperationNode::SetEvent(SetEventIRNode {
                        element,
                        key: self.expression(Expr::plain(name), true),
                        value: Some(self.expression(binding.value, false)),
                        modifiers,
                        delegate,
                        effect: false,
                    }));
            }
            BindingKind::Prop => {
                let mut values = Vec::new_in(&self.allocator);
                if let Some(merge) = binding.merge {
                    values.push(self.expression(Expr::plain(merge), true));
                }
                values.push(self.expression(binding.value, false));
                let key = self.expression(Expr::plain(binding.name), true);
                self.effect(
                    OperationNode::SetProp(SetPropIRNode {
                        element,
                        tag,
                        camel: false,
                        prop_modifier: false,
                        prop: IRProp {
                            key,
                            values,
                            is_component: false,
                        },
                    }),
                    block,
                );
            }
            BindingKind::Show => {
                let mut dir = DirectiveNode::new(self.allocator, "show", SourceLocation::STUB);
                dir.exp = Some(ExpressionNode::Simple(
                    self.expression(binding.value, false),
                ));
                block
                    .operation
                    .push(OperationNode::Directive(DirectiveIRNode {
                        element,
                        dir: Box::new_in(dir, &self.allocator),
                        name: "vShow",
                        builtin: true,
                        tag,
                        input_type,
                    }));
            }
            BindingKind::Html => {
                let value = self.expression(binding.value, false);
                self.effect(
                    OperationNode::SetHtml(SetHtmlIRNode { element, value }),
                    block,
                );
            }
            BindingKind::Text => {
                let values = self.values(binding.value);
                self.effect(
                    OperationNode::SetText(SetTextIRNode {
                        element,
                        is_element: true,
                        values,
                    }),
                    block,
                );
            }
        }
    }
}
