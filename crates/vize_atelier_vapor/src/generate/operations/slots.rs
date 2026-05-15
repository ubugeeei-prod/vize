use crate::ir::SlotOutletIRNode;
use vize_carton::cstr;

use super::super::context::GenerateContext;

/// Generate SlotOutlet
pub(super) fn generate_slot_outlet(ctx: &mut GenerateContext, slot: &SlotOutletIRNode<'_>) {
    let name = ctx.next_temp();
    let slot_name = if slot.name.is_static {
        cstr!("\"{}\"", slot.name.content)
    } else {
        vize_carton::CompactString::from(slot.name.content.as_str())
    };

    ctx.push_line_fmt(format_args!(
        "const {} = _renderSlot($slots, {})",
        name, slot_name
    ));
}
