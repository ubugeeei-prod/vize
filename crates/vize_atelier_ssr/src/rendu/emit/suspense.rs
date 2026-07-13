use vize_rendu::{RenduDynamicSlot, RenduName, RenduNode, RenduNodeId, RenduSlotPlan};

use super::SsrEmitter;

impl SsrEmitter<'_> {
    pub(super) fn emit_suspense(&mut self, children: &[RenduNodeId]) {
        let plan = self.root.component_slot_plan(children);
        self.indent();
        self.output.code.push_str("_ssrRenderSuspense(_push, ");
        if plan.has_dynamic_slots() {
            self.output.code.push_str("_createSlots(");
            self.emit_suspense_slot_base(&plan, true);
            self.output.code.push_str(", [");
            for (index, slot) in plan.dynamic_slots().iter().copied().enumerate() {
                if index > 0 {
                    self.output.code.push_str(", ");
                }
                self.emit_suspense_dynamic_slot(slot);
            }
            self.output.code.push_str("])");
        } else {
            self.emit_suspense_slot_base(&plan, false);
        }
        self.output.code.push_str(")\n");
    }

    fn emit_suspense_slot_base(&mut self, plan: &RenduSlotPlan, dynamic: bool) {
        self.output.code.push_str("{\n");
        self.indent += 1;
        let has_default = plan.static_slots().iter().any(|slot| {
            matches!(
                self.root.node(*slot),
                Some(RenduNode::SlotContent {
                    name: RenduName::Static(name),
                    ..
                }) if name.as_ref() == "default"
            )
        });
        if !plan.default_children().is_empty() || !has_default {
            self.emit_suspense_slot_property(
                &RenduName::static_name("default"),
                plan.default_children(),
            );
        }
        for &slot in plan.static_slots() {
            let Some(RenduNode::SlotContent { name, children, .. }) = self.root.node(slot) else {
                continue;
            };
            self.emit_suspense_slot_property(name, children);
        }
        self.indent();
        self.output
            .code
            .push_str(if dynamic { "_: 2\n" } else { "_: 1\n" });
        self.indent -= 1;
        self.indent();
        self.output.code.push('}');
    }

    fn emit_suspense_slot_property(&mut self, name: &RenduName, children: &[RenduNodeId]) {
        self.indent();
        self.emit_object_key(name);
        self.output.code.push_str(": ");
        self.emit_suspense_slot_function(children);
        self.output.code.push_str(",\n");
    }

    fn emit_suspense_slot_function(&mut self, children: &[RenduNodeId]) {
        self.output.code.push_str("() => {\n");
        self.indent += 1;
        self.emit_nodes(children);
        self.indent -= 1;
        self.indent();
        self.output.code.push('}');
    }

    fn emit_suspense_dynamic_slot(&mut self, slot: RenduDynamicSlot) {
        match slot {
            RenduDynamicSlot::Direct(slot) => self.emit_suspense_slot_descriptor(slot, None),
            RenduDynamicSlot::Conditional(node) => self.emit_suspense_conditional_slot(node),
            RenduDynamicSlot::Iterated(node) => self.emit_suspense_iterated_slot(node),
        }
    }

    fn emit_suspense_conditional_slot(&mut self, node: RenduNodeId) {
        let Some(RenduNode::If { branches, .. }) = self.root.node(node) else {
            self.output.code.push_str("undefined");
            return;
        };
        for (index, branch) in branches.iter().enumerate() {
            if index > 0 {
                self.output.code.push_str(" : ");
            }
            if let Some(condition) = branch.condition {
                self.output.code.push('(');
                self.emit_expression(condition);
                self.output.code.push_str(") ? ");
            }
            if let Some(slot) = self.root.slot_content_in(&branch.body) {
                self.emit_suspense_slot_descriptor(slot, Some(index));
            } else {
                self.output.code.push_str("undefined");
            }
        }
        if branches
            .last()
            .is_some_and(|branch| branch.condition.is_some())
        {
            self.output.code.push_str(" : undefined");
        }
    }

    fn emit_suspense_iterated_slot(&mut self, node: RenduNodeId) {
        let Some(RenduNode::For {
            source,
            value,
            key,
            index,
            body,
            ..
        }) = self.root.node(node)
        else {
            self.output.code.push_str("undefined");
            return;
        };
        let Some(slot) = self.root.slot_content_in(body) else {
            self.output.code.push_str("undefined");
            return;
        };
        self.output.code.push_str("_renderList(");
        self.emit_expression(*source);
        self.output.code.push_str(", (");
        self.emit_binding(value);
        for binding in [key.as_ref(), index.as_ref()].into_iter().flatten() {
            self.output.code.push_str(", ");
            self.emit_binding(binding);
        }
        self.output.code.push_str(") => { return ");
        self.emit_suspense_slot_descriptor(slot, None);
        self.output.code.push_str(" })");
    }

    fn emit_suspense_slot_descriptor(&mut self, slot: RenduNodeId, key: Option<usize>) {
        let Some(RenduNode::SlotContent { name, children, .. }) = self.root.node(slot) else {
            self.output.code.push_str("undefined");
            return;
        };
        self.output.code.push_str("{ name: ");
        self.emit_name_value(name);
        self.output.code.push_str(", fn: ");
        self.emit_suspense_slot_function(children);
        if let Some(key) = key {
            vize_carton::append!(self.output.code, ", key: \"{key}\"");
        }
        self.output.code.push_str(" }");
    }
}
