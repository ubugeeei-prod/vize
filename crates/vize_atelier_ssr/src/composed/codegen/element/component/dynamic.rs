// DO NOT EDIT. Generated from crates/vize_atelier_ssr/src/codegen/element/component/dynamic.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
//! Dynamic component SSR VNode emission.

use super::{ElementNode, RuntimeHelper, SsrCodegenContext, ToCompactString};

impl<'a> SsrCodegenContext<'a> {
    pub(super) fn process_dynamic_component(&mut self, el: &ElementNode<'a>, props: &str) {
        self.use_ssr_helper(RuntimeHelper::SsrRenderVNode);
        self.use_core_helper(RuntimeHelper::CreateVNode);

        let callee = self.dynamic_component_callee(el);
        let slots = if el.children.is_empty() {
            "null".to_compact_string()
        } else {
            self.vnode_component_slots_expression(el)
        };

        self.push_indent();
        self.push("_ssrRenderVNode(_push, _createVNode(");
        self.push(&callee);
        self.push(", ");
        self.push(props);
        self.push(", ");
        self.push(&slots);
        self.push("), _parent");
        if self.with_slot_scope_id {
            self.push(", _scopeId");
        }
        self.push(")\n");
    }
}
