//! Surface gates that are not a single provenance rule.

/// L1 freezes `v-pre` fallback mustaches before L2. The SSR plan cannot yet
/// prove the legacy rendering for a nonempty frozen fallback, even when L2
/// correctly contains only text. An empty fallback has nothing to freeze.
pub(super) fn slot_v_pre_has_fallback(source: &str, ops: &[vize_l2::op::Op<'_>]) -> bool {
    ops.iter().any(|op| match op {
        vize_l2::op::Op::Slot(slot) => {
            let bytes = source
                .get(slot.span.start as usize..slot.span.end as usize)
                .unwrap_or("");
            (bytes.contains("v-pre") && !slot.fallback.ops.is_empty())
                || slot_v_pre_has_fallback(source, &slot.fallback.ops)
        }
        vize_l2::op::Op::Element(element) => slot_v_pre_has_fallback(source, &element.children.ops),
        vize_l2::op::Op::Component(component) => {
            slot_v_pre_has_fallback(source, &component.children.ops)
        }
        vize_l2::op::Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| slot_v_pre_has_fallback(source, &branch.region.ops)),
        vize_l2::op::Op::For(for_op) => slot_v_pre_has_fallback(source, &for_op.region.ops),
        _ => false,
    })
}
