//! Surface gates that are not a single provenance rule.

/// `v-pre` on an outlet freezes fallback mustaches as text. An empty
/// fallback has nothing to freeze, so it renders like any other outlet.
pub(super) fn slot_v_pre_interpolates(source: &str, ops: &[vize_s2::op::Op<'_>]) -> bool {
    ops.iter().any(|op| match op {
        vize_s2::op::Op::Slot(slot) => {
            let bytes = source
                .get(slot.span.start as usize..slot.span.end as usize)
                .unwrap_or("");
            (bytes.contains("v-pre") && has_interpolation(&slot.fallback.ops))
                || slot_v_pre_interpolates(source, &slot.fallback.ops)
        }
        vize_s2::op::Op::Element(element) => slot_v_pre_interpolates(source, &element.children.ops),
        vize_s2::op::Op::Component(component) => {
            slot_v_pre_interpolates(source, &component.children.ops)
        }
        vize_s2::op::Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| slot_v_pre_interpolates(source, &branch.region.ops)),
        vize_s2::op::Op::For(for_op) => slot_v_pre_interpolates(source, &for_op.region.ops),
        _ => false,
    })
}

fn has_interpolation(ops: &[vize_s2::op::Op<'_>]) -> bool {
    ops.iter().any(|op| match op {
        vize_s2::op::Op::Interpolation(_) => true,
        vize_s2::op::Op::Element(element) => has_interpolation(&element.children.ops),
        vize_s2::op::Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| has_interpolation(&branch.region.ops)),
        vize_s2::op::Op::For(for_op) => has_interpolation(&for_op.region.ops),
        _ => false,
    })
}
