use vize_rendu::{
    RenduBinding, RenduDynamicSlot, RenduName, RenduNode, RenduNodeId, RenduSlotPlan,
};

use super::{
    DomEmitter,
    syntax::{comma, quote},
};

impl DomEmitter<'_> {
    pub(super) fn emit_component_slots(&mut self, children: &[RenduNodeId]) {
        let plan = self.root.component_slot_plan(children);
        if plan.has_dynamic_slots() {
            self.output.code.push_str("_createSlots(");
            self.emit_slot_base(&plan, true);
            self.output.code.push_str(", [");
            for (index, slot) in plan.dynamic_slots().iter().copied().enumerate() {
                if index > 0 {
                    self.output.code.push_str(", ");
                }
                self.emit_dynamic_slot(slot);
            }
            self.output.code.push_str("])");
        } else {
            self.emit_slot_base(&plan, false);
        }
    }

    fn emit_slot_base(&mut self, plan: &RenduSlotPlan, dynamic: bool) {
        self.output.code.push('{');
        let mut first = true;
        if !plan.default_children().is_empty() {
            comma(&mut self.output.code, &mut first);
            quote(&mut self.output.code, "default");
            self.output.code.push_str(": ");
            self.emit_slot_function(&[], plan.default_children());
        }
        for &slot in plan.static_slots() {
            comma(&mut self.output.code, &mut first);
            self.emit_slot_property(slot);
        }
        if dynamic {
            comma(&mut self.output.code, &mut first);
            self.output.code.push_str("_: 2");
        }
        self.output.code.push('}');
    }

    fn emit_slot_property(&mut self, slot: RenduNodeId) {
        let Some(RenduNode::SlotContent {
            name,
            bindings,
            children,
            ..
        }) = self.root.node(slot)
        else {
            return;
        };
        self.emit_slot_object_key(name);
        self.output.code.push_str(": ");
        self.emit_slot_function(bindings, children);
    }

    fn emit_slot_function(&mut self, bindings: &[RenduBinding], children: &[RenduNodeId]) {
        self.output.code.push_str("_withCtx((");
        for (index, binding) in bindings.iter().enumerate() {
            if index > 0 {
                self.output.code.push_str(", ");
            }
            self.output.code.push_str(&binding.pattern);
        }
        self.output.code.push_str(") => [");
        self.emit_node_list(children);
        self.output.code.push_str("])");
    }

    fn emit_dynamic_slot(&mut self, slot: RenduDynamicSlot) {
        match slot {
            RenduDynamicSlot::Direct(slot) => self.emit_slot_descriptor(slot, None),
            RenduDynamicSlot::Conditional(node) => self.emit_conditional_slot(node),
            RenduDynamicSlot::Iterated(node) => self.emit_iterated_slot(node),
        }
    }

    fn emit_conditional_slot(&mut self, node: RenduNodeId) {
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
                self.emit_slot_descriptor(slot, Some(index));
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

    fn emit_iterated_slot(&mut self, node: RenduNodeId) {
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
        self.output.code.push_str(&value.pattern);
        for binding in [key.as_ref(), index.as_ref()].into_iter().flatten() {
            self.output.code.push_str(", ");
            self.output.code.push_str(&binding.pattern);
        }
        self.output.code.push_str(") => { return ");
        self.emit_slot_descriptor(slot, None);
        self.output.code.push_str(" })");
    }

    fn emit_slot_descriptor(&mut self, slot: RenduNodeId, key: Option<usize>) {
        let Some(RenduNode::SlotContent {
            name,
            bindings,
            children,
            ..
        }) = self.root.node(slot)
        else {
            self.output.code.push_str("undefined");
            return;
        };
        self.output.code.push_str("{ name: ");
        self.emit_name_value(name);
        self.output.code.push_str(", fn: ");
        self.emit_slot_function(bindings, children);
        if let Some(key) = key {
            self.output.code.push_str(", key: \"");
            vize_carton::append!(self.output.code, "{key}");
            self.output.code.push('"');
        }
        self.output.code.push_str(" }");
    }

    fn emit_slot_object_key(&mut self, name: &RenduName) {
        match name {
            RenduName::Static(name) if vize_carton::is_simple_identifier(name) => {
                self.output.code.push_str(name);
            }
            RenduName::Static(name) => quote(&mut self.output.code, name),
            RenduName::Dynamic(expression) => {
                self.output.code.push('[');
                self.emit_expression(*expression);
                self.output.code.push(']');
            }
        }
    }
}
