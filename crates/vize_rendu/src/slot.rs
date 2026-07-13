//! Frontend-neutral interpretation of component children as Vue slot plans.

use crate::{RenduName, RenduNode, RenduNodeId, RenduRoot};

/// One dynamic entry consumed by Vue's `createSlots` runtime helper.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RenduDynamicSlot {
    /// A directly-authored slot with a runtime-computed name.
    Direct(RenduNodeId),
    /// An `if` whose branches select slot descriptors.
    Conditional(RenduNodeId),
    /// A `for` whose iterations produce slot descriptors.
    Iterated(RenduNodeId),
}

/// Classification of component children into the static slots object and its
/// optional dynamic `createSlots` entries.
///
/// This interpretation belongs to Rendu rather than an individual renderer so
/// DOM, SSR push rendering, and SSR's VNode fallback cannot disagree about
/// whether structural wrappers represent content or slot descriptors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduSlotPlan {
    default_children: Vec<RenduNodeId>,
    static_slots: Vec<RenduNodeId>,
    dynamic_slots: Vec<RenduDynamicSlot>,
}

impl RenduSlotPlan {
    pub fn new(root: &RenduRoot, children: &[RenduNodeId]) -> Self {
        let mut plan = Self {
            default_children: Vec::new(),
            static_slots: Vec::new(),
            dynamic_slots: Vec::new(),
        };
        for &child in children {
            match root.node(child) {
                Some(RenduNode::SlotContent {
                    name: RenduName::Static(_),
                    ..
                }) => plan.static_slots.push(child),
                Some(RenduNode::SlotContent {
                    name: RenduName::Dynamic(_),
                    ..
                }) => plan.dynamic_slots.push(RenduDynamicSlot::Direct(child)),
                Some(RenduNode::If { branches, .. })
                    if branches
                        .iter()
                        .any(|branch| root.slot_content_in(&branch.body).is_some()) =>
                {
                    plan.dynamic_slots
                        .push(RenduDynamicSlot::Conditional(child));
                }
                Some(RenduNode::For { body, .. }) if root.slot_content_in(body).is_some() => {
                    plan.dynamic_slots.push(RenduDynamicSlot::Iterated(child));
                }
                _ => plan.default_children.push(child),
            }
        }
        plan
    }

    pub fn default_children(&self) -> &[RenduNodeId] {
        &self.default_children
    }

    pub fn static_slots(&self) -> &[RenduNodeId] {
        &self.static_slots
    }

    pub fn dynamic_slots(&self) -> &[RenduDynamicSlot] {
        &self.dynamic_slots
    }

    pub const fn has_dynamic_slots(&self) -> bool {
        !self.dynamic_slots.is_empty()
    }
}

impl RenduRoot {
    /// Interpret a component's immediate children as one backend-neutral slot
    /// plan.
    pub fn component_slot_plan(&self, children: &[RenduNodeId]) -> RenduSlotPlan {
        RenduSlotPlan::new(self, children)
    }

    /// Find the first first-class slot descriptor directly contained by a
    /// structural wrapper body.
    pub fn slot_content_in(&self, nodes: &[RenduNodeId]) -> Option<RenduNodeId> {
        nodes
            .iter()
            .copied()
            .find(|id| matches!(self.node(*id), Some(RenduNode::SlotContent { .. })))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        RenduBinding, RenduBuilder, RenduComponentKind, RenduExpression, RenduExpressionKind,
        RenduIfBranch, RenduProvenance,
    };

    use super::*;

    #[test]
    fn classifies_static_dynamic_conditional_and_iterated_slots_once() {
        let mut builder = RenduBuilder::new();
        let dynamic_name =
            builder.add_expression(RenduExpression::new("name", RenduExpressionKind::Reference));
        let condition = builder.add_expression(RenduExpression::new(
            "enabled",
            RenduExpressionKind::Reference,
        ));
        let source = builder.add_expression(RenduExpression::new(
            "slots",
            RenduExpressionKind::Reference,
        ));
        let text = builder.add_node(RenduNode::Text {
            value: "ordinary".into(),
            provenance: RenduProvenance::generated(),
        });
        let static_slot = builder.add_node(RenduNode::SlotContent {
            name: RenduName::static_name("header"),
            bindings: Vec::new(),
            children: Vec::new(),
            provenance: RenduProvenance::generated(),
        });
        let direct_dynamic = builder.add_node(RenduNode::SlotContent {
            name: RenduName::Dynamic(dynamic_name),
            bindings: Vec::new(),
            children: Vec::new(),
            provenance: RenduProvenance::generated(),
        });
        let conditional_slot = builder.add_node(RenduNode::SlotContent {
            name: RenduName::static_name("conditional"),
            bindings: Vec::new(),
            children: Vec::new(),
            provenance: RenduProvenance::generated(),
        });
        let conditional = builder.add_node(RenduNode::If {
            branches: vec![RenduIfBranch::new(Some(condition), vec![conditional_slot])],
            provenance: RenduProvenance::generated(),
        });
        let iterated_slot = builder.add_node(RenduNode::SlotContent {
            name: RenduName::Dynamic(dynamic_name),
            bindings: Vec::new(),
            children: Vec::new(),
            provenance: RenduProvenance::generated(),
        });
        let iterated = builder.add_node(RenduNode::For {
            source,
            value: RenduBinding::new("slot"),
            key: None,
            index: None,
            key_expression: None,
            body: vec![iterated_slot],
            provenance: RenduProvenance::generated(),
        });
        let component = builder.add_node(RenduNode::Component {
            kind: RenduComponentKind::Ordinary,
            name: RenduName::static_name("Child"),
            properties: Vec::new(),
            children: vec![text, static_slot, direct_dynamic, conditional, iterated],
            provenance: RenduProvenance::generated(),
        });
        builder.push_entry(component);
        let root = builder.finish().expect("valid Rendu graph");
        let plan =
            root.component_slot_plan(&[text, static_slot, direct_dynamic, conditional, iterated]);

        assert_eq!(plan.default_children(), &[text]);
        assert_eq!(plan.static_slots(), &[static_slot]);
        assert_eq!(
            plan.dynamic_slots(),
            &[
                RenduDynamicSlot::Direct(direct_dynamic),
                RenduDynamicSlot::Conditional(conditional),
                RenduDynamicSlot::Iterated(iterated),
            ]
        );
    }
}
