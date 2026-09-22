//! Components and slot outlets. Slot content and fallbacks are blocks numbered
//! before their owner, as in the retained lane; props keep authored order.

use vize_atelier_core::{SimpleExpressionNode, SourceLocation};
use vize_carton::{Box, String, Vec};

use super::super::{BindingKind, Content, Expr, Node, Prop};
use super::{Emitter, take};
use crate::ir::{
    BlockIRNode, ComponentKind, CreateComponentIRNode, IRProp, IRSlot, OperationNode,
    SlotOutletIRNode,
};

impl<'a> Emitter<'a, '_> {
    /// `transform_component`: the default slot is built first, then the
    /// component takes its parent-assigned id or the next one.
    pub(super) fn component(
        &mut self,
        index: usize,
        existing: Option<usize>,
        placement: Option<(usize, usize)>,
        block: &mut BlockIRNode<'a>,
    ) {
        let Content::Component {
            tag,
            ref mut props,
            is,
        } = self.artifact.nodes[index].content
        else {
            unreachable!("component payload checked by the caller")
        };
        // Each node is emitted once, so its payload moves out of the artifact.
        let props = take(self.allocator, props);
        let children = take(self.allocator, &mut self.artifact.nodes[index].children);
        // Named templates each render their content block and then take the
        // template's own id; otherwise the children are one slot, `default`
        // unless the component's own `v-slot` names it.
        let mut slots = Vec::new_in(&self.allocator);
        let own = slot_of(&self.artifact.nodes[index]);
        let named = children
            .first()
            .is_some_and(|child| slot_of(&self.artifact.nodes[*child]).is_some());
        if named {
            for child in children {
                let slot = slot_of(&self.artifact.nodes[child]).expect("validated slot template");
                let content = take(self.allocator, &mut self.artifact.nodes[child].children);
                let block = self.block(&content);
                self.id();
                slots.push(self.slot(slot, block));
            }
        } else if own.is_some() || !children.is_empty() {
            let block = self.block(&children);
            slots.push(self.slot(own.unwrap_or(("default", "")), block));
        }
        let id = existing.unwrap_or_else(|| self.id());
        let props = self.props(&props, true);
        block
            .operation
            .push(OperationNode::CreateComponent(CreateComponentIRNode {
                id,
                tag,
                props,
                slots,
                asset: is.is_none(),
                once: false,
                dynamic_slots: false,
                kind: if is.is_some() {
                    ComponentKind::Dynamic
                } else {
                    ComponentKind::Regular
                },
                is_expr: is.map(|is| self.expression(is, false)),
                v_show: None,
                parent: placement.map(|(parent, _)| parent),
                anchor: placement.map(|(_, anchor)| anchor),
            }));
        if existing.is_none() {
            block.returns.push(id);
        }
    }

    /// A `<slot>` outlet with its already assigned id; the fallback block is
    /// numbered after it.
    pub(super) fn outlet(&mut self, index: usize, id: usize, block: &mut BlockIRNode<'a>) {
        let Content::Outlet {
            name,
            ref mut props,
        } = self.artifact.nodes[index].content
        else {
            unreachable!("outlet payload checked by the caller")
        };
        let props = take(self.allocator, props);
        let children = take(self.allocator, &mut self.artifact.nodes[index].children);
        let fallback = (!children.is_empty()).then(|| self.block(&children));
        let props = self.props(&props, false);
        let name = self.expression(Expr::plain(name), true);
        block
            .operation
            .push(OperationNode::SlotOutlet(SlotOutletIRNode {
                id,
                name,
                props,
                fallback,
            }));
    }

    /// One slot function: its static name, parameter pattern and block.
    fn slot(&self, (name, params): (&'a str, &'a str), block: BlockIRNode<'a>) -> IRSlot<'a> {
        IRSlot {
            name: self.expression(Expr::plain(name), true),
            fn_exp: (!params.is_empty()).then(|| self.expression(Expr::plain(params), false)),
            block,
        }
    }

    fn props(&self, props: &[Prop<'a>], component: bool) -> Vec<'a, IRProp<'a>> {
        let mut out = Vec::new_in(&self.allocator);
        for prop in props {
            let key = if prop.key == "$" {
                // A `v-bind`/`v-on` object source; `v-on` normalizes handlers.
                let mut node = SimpleExpressionNode::new("$", true, SourceLocation::STUB);
                node.is_handler_key = prop.handler;
                Box::new_in(node, &self.allocator)
            } else if prop.handler {
                // The retained lane's handler key: `on` + capitalized name.
                let mut key = String::from("on");
                let mut chars = prop.key.chars();
                if let Some(first) = chars.next() {
                    key.push(first.to_ascii_uppercase());
                    key.push_str(chars.as_str());
                }
                let mut node = SimpleExpressionNode::new(
                    self.allocator.alloc_str(&key),
                    true,
                    SourceLocation::STUB,
                );
                node.is_handler_key = true;
                Box::new_in(node, &self.allocator)
            } else {
                self.expression(Expr::plain(prop.key), true)
            };
            let mut values = Vec::new_in(&self.allocator);
            if let Some(value) = prop.value {
                values.push(self.expression(value, !prop.dynamic));
            }
            out.push(IRProp {
                key,
                values,
                is_component: component,
            });
        }
        out
    }
}

/// The slot a template or component binds: its name and parameter pattern.
fn slot_of<'a>(node: &Node<'a>) -> Option<(&'a str, &'a str)> {
    node.bindings
        .iter()
        .find(|binding| binding.kind == BindingKind::Slot)
        .map(|binding| (binding.name, binding.value.text))
}
