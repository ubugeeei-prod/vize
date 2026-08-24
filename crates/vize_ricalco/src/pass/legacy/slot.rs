//! `vue.slot-scope` → `ui.slot-content` 1:1 (same introduction site).

use vize_s0::{Allocator, Box, Vec};
use vize_s2::op::{BindingOp, DynamicName, SlotContentOp, VueSlotScopeOp};

/// Replace every `vue.slot-scope` with `ui.slot-content`. Scope facts
/// already key this binding; the rewrite does not mint a new id of its
/// own (any shift is `.sync` insertion, rekeyed after).
pub(super) fn convert<'a>(allocator: &'a Allocator, bindings: &mut Vec<'a, BindingOp<'a>>) {
    for binding in bindings.iter_mut() {
        let BindingOp::VueSlotScope(scope) = binding else {
            continue;
        };
        *binding = to_slot_content(allocator, scope);
    }
}

fn to_slot_content<'a>(allocator: &'a Allocator, scope: &VueSlotScopeOp<'a>) -> BindingOp<'a> {
    BindingOp::SlotContent(Box::new_in(
        SlotContentOp {
            name: scope.name.map(DynamicName::Static),
            modifiers: Vec::new_in(&allocator),
            params: scope.params,
            span: scope.span,
        },
        &allocator,
    ))
}
