//! Preserve Rendu's slot classification in the owned Vapor plan.

use vize_rendu::{RenduDynamicSlot, RenduNode, RenduNodeId, RenduProvenance};

use crate::rendu::{VaporComponentSlots, VaporConditionalSlotBranch, VaporDynamicSlot, VaporSlot};

use super::{Planner, expression_id, lower_binding, lower_name};

impl Planner<'_> {
    pub(super) fn lower_component_slots(
        &mut self,
        children: &[RenduNodeId],
        provenance: RenduProvenance,
    ) -> VaporComponentSlots {
        let plan = self.root.component_slot_plan(children);
        let default = (!plan.default_children().is_empty()
            || (plan.static_slots().is_empty() && plan.dynamic_slots().is_empty()))
        .then(|| self.lower_block(plan.default_children(), provenance));
        let static_slots = plan
            .static_slots()
            .iter()
            .copied()
            .filter_map(|slot| self.lower_slot(slot))
            .collect();
        let dynamic_slots = plan
            .dynamic_slots()
            .iter()
            .copied()
            .filter_map(|slot| self.lower_dynamic_slot(slot))
            .collect();
        VaporComponentSlots {
            default,
            static_slots,
            dynamic_slots,
        }
    }

    fn lower_dynamic_slot(&mut self, slot: RenduDynamicSlot) -> Option<VaporDynamicSlot> {
        match slot {
            RenduDynamicSlot::Direct(slot) => self.lower_slot(slot).map(VaporDynamicSlot::Direct),
            RenduDynamicSlot::Conditional(node) => {
                let RenduNode::If {
                    branches,
                    provenance,
                } = self.root.node(node)?
                else {
                    return None;
                };
                let provenance = provenance.clone();
                let branches = branches
                    .iter()
                    .map(|branch| {
                        (
                            branch.condition.map(expression_id),
                            self.root.slot_content_in(&branch.body),
                            branch.provenance.clone(),
                        )
                    })
                    .collect::<Vec<_>>();
                Some(VaporDynamicSlot::Conditional {
                    branches: branches
                        .into_iter()
                        .map(|(condition, slot, provenance)| VaporConditionalSlotBranch {
                            condition,
                            slot: slot.and_then(|slot| self.lower_slot(slot)),
                            provenance,
                        })
                        .collect(),
                    provenance,
                })
            }
            RenduDynamicSlot::Iterated(node) => {
                let RenduNode::For {
                    source,
                    value,
                    key,
                    index,
                    body,
                    provenance,
                    ..
                } = self.root.node(node)?
                else {
                    return None;
                };
                let source = expression_id(*source);
                let value = lower_binding(value);
                let key = key.as_ref().map(lower_binding);
                let index = index.as_ref().map(lower_binding);
                let slot = self.root.slot_content_in(body)?;
                let provenance = provenance.clone();
                self.lower_slot(slot)
                    .map(|slot| VaporDynamicSlot::Iterated {
                        source,
                        value,
                        key,
                        index,
                        slot: Box::new(slot),
                        provenance,
                    })
            }
        }
    }

    fn lower_slot(&mut self, id: RenduNodeId) -> Option<VaporSlot> {
        let RenduNode::SlotContent {
            name,
            bindings,
            children,
            provenance,
        } = self.root.node(id)?
        else {
            return None;
        };
        let name = lower_name(name);
        let bindings = bindings.iter().map(lower_binding).collect();
        let children = children.clone();
        let provenance = provenance.clone();
        let body = self.lower_block(&children, provenance.clone());
        Some(VaporSlot {
            name,
            bindings,
            body,
            provenance,
        })
    }
}
