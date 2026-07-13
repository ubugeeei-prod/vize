use vize_rendu::{RenduBinding, RenduDynamicSlot, RenduNode, RenduNodeId, RenduSlotPlan};

use super::SsrEmitter;

impl SsrEmitter<'_> {
    pub(super) fn emit_slots(&mut self, children: &[RenduNodeId]) {
        let plan = self.root.component_slot_plan(children);
        if plan.has_dynamic_slots() {
            self.output.code.push_str("_createSlots(");
            self.emit_ssr_slot_base(&plan, true);
            self.output.code.push_str(", [");
            for (index, slot) in plan.dynamic_slots().iter().copied().enumerate() {
                if index > 0 {
                    self.output.code.push_str(", ");
                }
                self.emit_ssr_dynamic_slot(slot);
            }
            self.output.code.push_str("])");
        } else {
            self.emit_ssr_slot_base(&plan, false);
        }
    }

    fn emit_ssr_slot_base(&mut self, plan: &RenduSlotPlan, dynamic: bool) {
        self.output.code.push('{');
        let mut wrote = false;
        if !plan.default_children().is_empty() {
            self.output.code.push_str(" default: ");
            self.emit_ssr_slot_function(&[], plan.default_children());
            wrote = true;
        }
        for &slot in plan.static_slots() {
            if wrote {
                self.output.code.push(',');
            }
            self.output.code.push(' ');
            self.emit_ssr_slot_property(slot);
            wrote = true;
        }
        if dynamic {
            if wrote {
                self.output.code.push(',');
            }
            self.output.code.push_str(" _: 2");
            wrote = true;
        }
        if wrote {
            self.output.code.push(' ');
        }
        self.output.code.push('}');
    }

    fn emit_ssr_slot_property(&mut self, slot: RenduNodeId) {
        let Some(RenduNode::SlotContent {
            name,
            bindings,
            children,
            ..
        }) = self.root.node(slot)
        else {
            return;
        };
        self.emit_object_key(name);
        self.output.code.push_str(": ");
        self.emit_ssr_slot_function(bindings, children);
    }

    fn emit_ssr_slot_function(&mut self, bindings: &[RenduBinding], children: &[RenduNodeId]) {
        self.output.code.push_str("_withCtx((");
        if bindings.is_empty() {
            self.output.code.push_str("_props");
        } else {
            for (index, binding) in bindings.iter().enumerate() {
                if index > 0 {
                    self.output.code.push_str(", ");
                }
                self.emit_binding(binding);
            }
        }
        self.output
            .code
            .push_str(", _push, _parent, _scopeId) => {\n");
        self.indent += 1;
        self.indent();
        self.output.code.push_str("if (_push) {\n");
        self.indent += 1;
        self.slot_scope_depth += 1;
        self.emit_nodes(children);
        self.slot_scope_depth -= 1;
        self.indent -= 1;
        self.indent();
        self.output.code.push_str("} else {\n");
        self.indent += 1;
        self.indent();
        self.output.code.push_str("return [");
        self.emit_vnode_list(children);
        self.output.code.push_str("]\n");
        self.indent -= 1;
        self.indent();
        self.output.code.push_str("}\n");
        self.indent -= 1;
        self.indent();
        self.output.code.push_str("})");
    }

    fn emit_ssr_dynamic_slot(&mut self, slot: RenduDynamicSlot) {
        match slot {
            RenduDynamicSlot::Direct(slot) => self.emit_ssr_slot_descriptor(slot, None),
            RenduDynamicSlot::Conditional(node) => self.emit_ssr_conditional_slot(node),
            RenduDynamicSlot::Iterated(node) => self.emit_ssr_iterated_slot(node),
        }
    }

    fn emit_ssr_conditional_slot(&mut self, node: RenduNodeId) {
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
                self.emit_ssr_slot_descriptor(slot, Some(index));
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

    fn emit_ssr_iterated_slot(&mut self, node: RenduNodeId) {
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
        self.emit_ssr_slot_descriptor(slot, None);
        self.output.code.push_str(" })");
    }

    fn emit_ssr_slot_descriptor(&mut self, slot: RenduNodeId, key: Option<usize>) {
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
        self.emit_ssr_slot_function(bindings, children);
        if let Some(key) = key {
            vize_carton::append!(self.output.code, ", key: \"{key}\"");
        }
        self.output.code.push_str(" }");
    }
}
