//! Checked component slot carriers become expression metadata, never DOM control.

use super::super::{Branch, Content};
use super::{Emitter, component::slot_of, take};
use crate::ir::{IRSlot, IRSlotControl, IRSlotLoop};
use vize_carton::Box;

impl<'a> Emitter<'a, '_> {
    pub(super) fn component_slot(&mut self, index: usize) -> Option<IRSlot<'a>> {
        let Some(node) = self.artifact.nodes.get_mut(index) else {
            self.invariant_broken();
            return None;
        };
        match &mut node.content {
            Content::Element {
                tag: "template", ..
            } => {
                let Some(slot) = slot_of(node) else {
                    self.invariant_broken();
                    return None;
                };
                let members = take(self.allocator, &mut node.children);
                let block = self.block(&members);
                self.id();
                Some(self.slot(index, Some(slot), block))
            }
            Content::If { branches } => {
                if !branches
                    .first()
                    .is_some_and(|branch| branch.condition.is_some())
                {
                    self.invariant_broken();
                    return None;
                }
                let branches = take(self.allocator, branches);
                self.conditional_slot(&branches)
            }
            Content::For(body) => {
                let body = *body;
                let [child] = node.children.as_slice() else {
                    self.invariant_broken();
                    return None;
                };
                let child = *child;
                let mut slot = self.component_slot(child)?;
                let alias = |value, span: Option<super::super::AuthoredSpan>| {
                    self.spanned(
                        super::super::Expr::plain(value),
                        false,
                        span.map(|span| self.trimmed(span)),
                    )
                };
                slot.control = Some(IRSlotControl::For(IRSlotLoop {
                    source: self.spanned(body.source, false, Some(self.trimmed(body.spans.source))),
                    value: alias(body.value, body.spans.aliases[0]),
                    key: body.key.map(|key| alias(key, body.spans.aliases[1])),
                    index: body.index.map(|index| alias(index, body.spans.aliases[2])),
                    key_prop: body.key_prop.map(|key| {
                        self.spanned(
                            key,
                            false,
                            body.spans.key_prop.map(|span| self.trimmed(span)),
                        )
                    }),
                }));
                Some(slot)
            }
            _ => {
                self.invariant_broken();
                None
            }
        }
    }

    fn conditional_slot(&mut self, branches: &[Branch<'a>]) -> Option<IRSlot<'a>> {
        let (branch, rest) = branches.split_first()?;
        let [child] = branch.roots.as_slice() else {
            self.invariant_broken();
            return None;
        };
        let mut slot = self.component_slot(*child)?;
        if branch.condition.is_none() && !rest.is_empty() {
            self.invariant_broken();
            return None;
        }
        if let Some(condition) = branch.condition {
            let negative = self
                .conditional_slot(rest)
                .map(|slot| Box::new_in(slot, &self.allocator));
            slot.control = Some(IRSlotControl::If {
                condition: self.spanned(condition, false, Some(self.trimmed(branch.span))),
                negative,
            });
        }
        Some(slot)
    }
}
