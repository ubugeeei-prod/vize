//! Dynamic component SSR VNode emission.

use vize_atelier_core::codegen::RenderMapMode;
use super::{ElementNode, RuntimeHelper, SsrCodegenContext, ToCompactString};

impl<'a, M: RenderMapMode> SsrCodegenContext<'a, M> {
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
