//! SSR slot property and `_withCtx` slot function emission.
//!
//! Slot names map to their authored `v-slot` argument and each slot function
//! opens at the authored element carrying the slot (Davinci P3-9).

use super::props::{is_valid_js_identifier, quoted_js_string};
use super::{
    ComponentSlotChildren, FxHashSet, RuntimeHelper, SlotAnchor, SsrCodegenContext, String,
};

impl<'a> SsrCodegenContext<'a> {
    pub(super) fn process_component_slot_property<'node>(
        &mut self,
        name: &str,
        anchor: Option<SlotAnchor>,
        props_pattern: Option<&str>,
        params: &FxHashSet<String>,
        children: ComponentSlotChildren<'node, 'a>,
    ) {
        self.push_indent();
        let name_start = anchor.and_then(|anchor| anchor.name);
        if is_valid_js_identifier(name) {
            self.push_optionally_mapped(name, name_start);
        } else {
            let quoted = quoted_js_string(name);
            self.push("\"");
            self.push_optionally_mapped(
                quoted.get(1..quoted.len() - 1).unwrap_or_default(),
                name_start,
            );
            self.push("\"");
        }
        self.push(": ");
        self.emit_slot_fn(
            anchor.map(|anchor| anchor.unit),
            props_pattern,
            params,
            children,
        );
        self.push(",\n");
    }

    /// Emit a `_withCtx((params, _push, _parent, _scopeId) => { if (_push) {...}
    /// else { return [...] } })` slot function, shared by static slot properties
    /// and `createSlots` entries.
    pub(super) fn emit_slot_fn<'node>(
        &mut self,
        unit: Option<u32>,
        props_pattern: Option<&str>,
        params: &FxHashSet<String>,
        children: ComponentSlotChildren<'node, 'a>,
    ) {
        self.use_core_helper(RuntimeHelper::WithCtx);
        // The slot function opens at the authored unit carrying the slot.
        self.push_optionally_mapped("_withCtx((", unit);
        self.push(props_pattern.unwrap_or("_"));
        self.push(", _push, _parent, _scopeId) => {\n");
        self.indent_level += 1;
        self.push_indent();
        self.push("if (_push) {\n");
        self.indent_level += 1;

        let old_parts = std::mem::take(&mut self.current_template_parts);
        let previous_slot_scope = self.with_slot_scope_id;
        self.with_slot_scope_id = true;
        if !params.is_empty() {
            self.push_scoped_params(params.clone());
        }
        self.process_component_slot_children(&children);
        self.flush_push();
        if !params.is_empty() {
            self.pop_scoped_params();
        }
        self.with_slot_scope_id = previous_slot_scope;
        self.current_template_parts = old_parts;

        self.indent_level -= 1;
        self.push_indent();
        self.push("} else {\n");
        self.indent_level += 1;
        if !params.is_empty() {
            self.push_scoped_params(params.clone());
        }
        let fallback = self.vnode_component_slot_children_expression(&children);
        if !params.is_empty() {
            self.pop_scoped_params();
        }
        self.push_indent();
        self.push("return ");
        self.push(&fallback);
        self.push("\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("}\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("})");
    }
}
