//! Components and slot outlets. Slot content and fallbacks are blocks numbered
//! before their owner, as in the retained lane; props keep authored order.

use vize_atelier_core::{SimpleExpressionNode, SourceLocation};
use vize_carton::{Box, String, Vec};

use super::super::{Content, Expr, Prop};
use super::Emitter;
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
        let Content::Component { tag, ref props } = self.artifact.nodes[index].content else {
            unreachable!("component payload checked by the caller")
        };
        let props = props.clone();
        let children = self.artifact.nodes[index].children.clone();
        let mut slots = Vec::new_in(&self.allocator);
        if !children.is_empty() {
            let slot = self.block(&children);
            slots.push(IRSlot {
                name: self.expression(Expr::plain("default"), true),
                fn_exp: None,
                block: slot,
            });
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
                asset: true,
                once: false,
                dynamic_slots: false,
                kind: ComponentKind::Regular,
                is_expr: None,
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
        let Content::Outlet { name, ref props } = self.artifact.nodes[index].content else {
            unreachable!("outlet payload checked by the caller")
        };
        let props = props.clone();
        let children = self.artifact.nodes[index].children.clone();
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

    fn props(&self, props: &[Prop<'a>], component: bool) -> Vec<'a, IRProp<'a>> {
        let mut out = Vec::new_in(&self.allocator);
        for prop in props {
            let key = if prop.handler {
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
