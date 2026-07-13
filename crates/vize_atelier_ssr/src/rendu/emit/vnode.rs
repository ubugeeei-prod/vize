use vize_rendu::{
    RenduBinding, RenduDynamicSlot, RenduEscapeMode, RenduName, RenduNode, RenduNodeId,
    RenduSlotPlan,
};

use super::{SsrEmitter, syntax::quote_js};

impl SsrEmitter<'_> {
    pub(super) fn emit_vnode_list(&mut self, nodes: &[RenduNodeId]) {
        for (index, node) in nodes.iter().enumerate() {
            if index > 0 {
                self.output.code.push_str(", ");
            }
            self.emit_vnode(*node);
        }
    }

    fn emit_vnode(&mut self, id: RenduNodeId) {
        match self.root.node(id).expect("validated VNode slot child") {
            RenduNode::Fragment { children, .. } | RenduNode::SlotContent { children, .. } => {
                self.emit_vnode_group(children);
            }
            RenduNode::Element {
                tag,
                properties,
                children,
                ..
            } => {
                self.output.code.push_str("_createVNode(");
                quote_js(&mut self.output.code, tag);
                self.output.code.push_str(", ");
                self.emit_properties(properties);
                self.output.code.push_str(", [");
                self.emit_vnode_list(children);
                self.output.code.push_str("])");
            }
            RenduNode::Component {
                kind,
                name,
                properties,
                children,
                ..
            } => {
                self.output.code.push_str("_createVNode(");
                self.emit_component_name(*kind, name, properties);
                self.output.code.push_str(", ");
                self.emit_component_properties(*kind, properties);
                self.output.code.push_str(", ");
                self.emit_vnode_slots(children);
                self.output.code.push(')');
            }
            RenduNode::SlotOutlet {
                name,
                properties,
                fallback,
                ..
            } => {
                self.output.code.push_str("_renderSlot(_ctx.$slots, ");
                self.emit_name_value(name);
                self.output.code.push_str(", ");
                self.emit_properties(properties);
                self.output.code.push_str(", () => [");
                self.emit_vnode_list(fallback);
                self.output.code.push_str("])");
            }
            RenduNode::Text { value, .. } => {
                self.output.code.push_str("_createTextVNode(");
                quote_js(&mut self.output.code, value);
                self.output.code.push(')');
            }
            RenduNode::Expression {
                expression, escape, ..
            } => {
                self.output.code.push_str("_createTextVNode(");
                if matches!(escape, RenduEscapeMode::Escaped) {
                    self.output.code.push_str("_toDisplayString(");
                }
                self.emit_expression(*expression);
                if matches!(escape, RenduEscapeMode::Escaped) {
                    self.output.code.push(')');
                }
                self.output.code.push(')');
            }
            RenduNode::Comment { value, .. } => {
                self.output.code.push_str("_createCommentVNode(");
                quote_js(&mut self.output.code, value);
                self.output.code.push(')');
            }
            RenduNode::If { branches, .. } => self.emit_vnode_if(branches),
            RenduNode::For {
                source,
                value,
                key,
                index,
                body,
                ..
            } => {
                self.output.code.push_str("_renderList(");
                self.emit_expression(*source);
                self.output.code.push_str(", (");
                self.emit_binding(value);
                for binding in [key.as_ref(), index.as_ref()].into_iter().flatten() {
                    self.output.code.push_str(", ");
                    self.emit_binding(binding);
                }
                self.output.code.push_str(") => ");
                self.emit_vnode_group(body);
                self.output.code.push(')');
            }
            RenduNode::HoistRef { index, .. } => {
                vize_carton::append!(self.output.code, "_ctx._hoisted?.[{index}] ?? null");
            }
            _ => unreachable!("RenduNode is non-exhaustive across backend crates"),
        }
    }

    fn emit_vnode_group(&mut self, nodes: &[RenduNodeId]) {
        if let [node] = nodes {
            self.emit_vnode(*node);
            return;
        }
        self.output.code.push_str("_createVNode(_Fragment, null, [");
        self.emit_vnode_list(nodes);
        self.output.code.push_str("])");
    }

    fn emit_vnode_if(&mut self, branches: &[vize_rendu::RenduIfBranch]) {
        for (index, branch) in branches.iter().enumerate() {
            if let Some(condition) = branch.condition {
                self.output.code.push('(');
                self.emit_expression(condition);
                self.output.code.push_str(") ? ");
            }
            self.emit_vnode_group(&branch.body);
            if index + 1 < branches.len() {
                self.output.code.push_str(" : ");
            }
        }
        if branches
            .last()
            .is_some_and(|branch| branch.condition.is_some())
        {
            self.output.code.push_str(" : null");
        }
    }

    fn emit_vnode_slots(&mut self, children: &[RenduNodeId]) {
        let plan = self.root.component_slot_plan(children);
        if plan.has_dynamic_slots() {
            self.output.code.push_str("_createSlots(");
            self.emit_vnode_slot_base(&plan, true);
            self.output.code.push_str(", [");
            for (index, slot) in plan.dynamic_slots().iter().copied().enumerate() {
                if index > 0 {
                    self.output.code.push_str(", ");
                }
                self.emit_vnode_dynamic_slot(slot);
            }
            self.output.code.push_str("])");
        } else {
            self.emit_vnode_slot_base(&plan, false);
        }
    }

    fn emit_vnode_slot_base(&mut self, plan: &RenduSlotPlan, dynamic: bool) {
        self.output.code.push('{');
        let mut wrote = false;
        if !plan.default_children().is_empty() {
            self.output.code.push_str(" default: ");
            self.emit_vnode_slot_function(&[], plan.default_children());
            wrote = true;
        }
        for &slot in plan.static_slots() {
            if wrote {
                self.output.code.push(',');
            }
            self.output.code.push(' ');
            self.emit_vnode_slot_property(slot);
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

    fn emit_vnode_slot_property(&mut self, slot: RenduNodeId) {
        let Some(RenduNode::SlotContent {
            name,
            bindings,
            children,
            ..
        }) = self.root.node(slot)
        else {
            return;
        };
        match name {
            RenduName::Static(name) => quote_js(&mut self.output.code, name),
            RenduName::Dynamic(expression) => {
                self.output.code.push('[');
                self.emit_expression(*expression);
                self.output.code.push(']');
            }
        }
        self.output.code.push_str(": ");
        self.emit_vnode_slot_function(bindings, children);
    }

    fn emit_vnode_slot_function(&mut self, bindings: &[RenduBinding], children: &[RenduNodeId]) {
        self.output.code.push_str("_withCtx((");
        for (index, binding) in bindings.iter().enumerate() {
            if index > 0 {
                self.output.code.push_str(", ");
            }
            self.emit_binding(binding);
        }
        self.output.code.push_str(") => [");
        self.emit_vnode_list(children);
        self.output.code.push_str("])");
    }

    fn emit_vnode_dynamic_slot(&mut self, slot: RenduDynamicSlot) {
        match slot {
            RenduDynamicSlot::Direct(slot) => self.emit_vnode_slot_descriptor(slot, None),
            RenduDynamicSlot::Conditional(node) => self.emit_vnode_conditional_slot(node),
            RenduDynamicSlot::Iterated(node) => self.emit_vnode_iterated_slot(node),
        }
    }

    fn emit_vnode_conditional_slot(&mut self, node: RenduNodeId) {
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
                self.emit_vnode_slot_descriptor(slot, Some(index));
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

    fn emit_vnode_iterated_slot(&mut self, node: RenduNodeId) {
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
        self.emit_vnode_slot_descriptor(slot, None);
        self.output.code.push_str(" })");
    }

    fn emit_vnode_slot_descriptor(&mut self, slot: RenduNodeId, key: Option<usize>) {
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
        self.emit_vnode_slot_function(bindings, children);
        if let Some(key) = key {
            vize_carton::append!(self.output.code, ", key: \"{key}\"");
        }
        self.output.code.push_str(" }");
    }
}
