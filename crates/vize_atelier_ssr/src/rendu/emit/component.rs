use vize_rendu::{RenduName, RenduNode, RenduNodeId, RenduProperty};

use super::SsrEmitter;

impl SsrEmitter<'_> {
    pub(super) fn emit_component(
        &mut self,
        name: &RenduName,
        properties: &[RenduProperty],
        children: &[RenduNodeId],
    ) {
        self.indent();
        self.output.code.push_str("_push(_ssrRenderComponent(");
        self.emit_component_name(name);
        self.output.code.push_str(", ");
        self.emit_properties(properties);
        self.output.code.push_str(", ");
        self.emit_slots(children);
        self.output.code.push_str(", _parent))\n");
    }

    pub(super) fn emit_slot_outlet(
        &mut self,
        name: &RenduName,
        properties: &[RenduProperty],
        fallback: &[RenduNodeId],
    ) {
        self.indent();
        self.output.code.push_str("_ssrRenderSlot(_ctx.$slots, ");
        self.emit_name_value(name);
        self.output.code.push_str(", ");
        self.emit_properties(properties);
        self.output.code.push_str(", () => {\n");
        self.indent += 1;
        self.emit_nodes(fallback);
        self.indent -= 1;
        self.indent();
        self.output.code.push_str("}, _push, _parent)\n");
    }

    fn emit_slots(&mut self, children: &[RenduNodeId]) {
        self.output.code.push('{');
        let mut wrote = false;
        let ordinary: Vec<_> = children
            .iter()
            .copied()
            .filter(|id| {
                !matches!(
                    self.root.node(*id).expect("validated slot child"),
                    RenduNode::SlotContent { .. }
                )
            })
            .collect();
        if !ordinary.is_empty() {
            self.output
                .code
                .push_str(" default: (_props, _push, _parent) => {\n");
            self.indent += 1;
            self.emit_nodes(&ordinary);
            self.indent -= 1;
            self.indent();
            self.output.code.push('}');
            wrote = true;
        }
        for &child in children {
            let Some(RenduNode::SlotContent {
                name,
                bindings,
                children,
                ..
            }) = self.root.node(child)
            else {
                continue;
            };
            if wrote {
                self.output.code.push(',');
            }
            self.output.code.push(' ');
            self.emit_object_key(name);
            self.output.code.push_str(": (");
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
            self.output.code.push_str(", _push, _parent) => {\n");
            self.indent += 1;
            self.emit_nodes(children);
            self.indent -= 1;
            self.indent();
            self.output.code.push('}');
            wrote = true;
        }
        if wrote {
            self.output.code.push(' ');
        }
        self.output.code.push('}');
    }
}
