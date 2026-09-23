//! Element bindings in authored order: props and content directives are
//! render effects; listeners and `v-show` are one-time operations.

use vize_atelier_core::{DirectiveNode, ExpressionNode, SimpleExpressionNode, SourceLocation};
use vize_carton::{Box, Vec};

use super::super::{BindingKind, Content, Expr};
use super::Emitter;
use super::spans::argument_offset;
use crate::ir::{
    BlockIRNode, DirectiveIRNode, EventModifiers, IRProp, OperationNode, SetDynamicPropsIRNode,
    SetEventIRNode, SetHtmlIRNode, SetPropIRNode, SetTextIRNode,
};

impl<'a> Emitter<'a, '_> {
    /// Every binding of an element, in authored order.
    pub(super) fn bindings(&mut self, index: usize, element: usize, block: &mut BlockIRNode<'a>) {
        let Some(node) = self.artifact.nodes.get(index) else {
            return self.invariant_broken();
        };
        if node.bindings.iter().any(|b| b.kind == BindingKind::Spread) {
            return self.merged_props(index, element, block);
        }
        for binding in 0..node.bindings.len() {
            self.binding(index, binding, element, block);
        }
    }

    fn binding(
        &mut self,
        index: usize,
        binding: usize,
        element: usize,
        block: &mut BlockIRNode<'a>,
    ) {
        // Bindings attach to elements.
        let Some(node) = self.artifact.nodes.get(index) else {
            return self.invariant_broken();
        };
        let Content::Element {
            tag,
            ref attributes,
            ..
        } = node.content
        else {
            return self.invariant_broken();
        };
        let input_type = attributes
            .iter()
            .find_map(|(name, value, _)| (*name == "type").then_some(*value).flatten())
            .unwrap_or("");
        let Some(binding) = node.bindings.get(binding) else {
            return self.invariant_broken();
        };
        // Values keep their authored span; a prop key maps to its argument.
        let [name_raw, value_raw] = binding.spans;
        let value_span = Some(self.trimmed(value_raw));
        let name_span = self.token(name_raw, |raw| argument_offset(raw, binding.name));
        match binding.kind {
            BindingKind::Cloak => {
                let dir = DirectiveNode::new(self.allocator, "cloak", SourceLocation::STUB);
                block
                    .operation
                    .push(OperationNode::Directive(DirectiveIRNode {
                        element,
                        dir: Box::new_in(dir, &self.allocator),
                        name: "vCloak",
                        builtin: true,
                        tag,
                        input_type,
                    }));
            }
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
                        value: Some(self.spanned(binding.value, false, value_span)),
                        modifiers,
                        delegate,
                        effect: false,
                    }));
            }
            BindingKind::Prop => {
                let mut values = Vec::new_in(&self.allocator);
                if let Some((merge, false)) = binding.merge {
                    values.push(self.expression(Expr::plain(merge), true));
                }
                values.push(self.spanned(binding.value, false, value_span));
                if let Some((merge, true)) = binding.merge {
                    values.push(self.expression(Expr::plain(merge), true));
                }
                let key = self.spanned(Expr::plain(binding.name), true, name_span);
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
                dir.exp = Some(ExpressionNode::Simple(self.spanned(
                    binding.value,
                    false,
                    value_span,
                )));
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
                let value = self.spanned(binding.value, false, value_span);
                self.effect(
                    OperationNode::SetHtml(SetHtmlIRNode { element, value }),
                    block,
                );
            }
            BindingKind::Text => {
                let values = self.values(binding.value, value_span);
                self.effect(
                    OperationNode::SetText(SetTextIRNode {
                        element,
                        is_element: true,
                        values,
                    }),
                    block,
                );
            }
            BindingKind::Model => {
                let mut dir = DirectiveNode::new(self.allocator, "model", SourceLocation::STUB);
                dir.exp = Some(ExpressionNode::Simple(self.spanned(
                    binding.value,
                    false,
                    value_span,
                )));
                for modifier in &binding.modifiers {
                    dir.modifiers.push(SimpleExpressionNode::new(
                        modifier,
                        true,
                        SourceLocation::STUB,
                    ));
                }
                block
                    .operation
                    .push(OperationNode::Directive(DirectiveIRNode {
                        element,
                        dir: Box::new_in(dir, &self.allocator),
                        name: "model",
                        builtin: true,
                        tag,
                        input_type,
                    }));
            }
            BindingKind::Handlers => {
                let props = self.values(binding.value, value_span);
                self.effect(
                    OperationNode::SetDynamicProps(SetDynamicPropsIRNode {
                        element,
                        props,
                        is_event: true,
                    }),
                    block,
                );
            }
            // Slot content is emitted with its component, and a spread
            // element merges its props instead.
            BindingKind::Slot | BindingKind::Spread => self.invariant_broken(),
        }
    }
}
