use crate::ir::InsertionAnchor;
use vize_carton::cstr;

use super::super::context::GenerateContext;

/// Emit Vue 3.6's `setInsertionState(parent, anchor?)` before a block that
/// needs insertion. A node anchor is the template placeholder the block is
/// inserted before; an index anchor appends and carries the hydration start
/// unit, omitted when it is zero exactly as upstream codegen does.
pub(super) fn emit_insertion_state(
    ctx: &mut GenerateContext,
    parent: Option<usize>,
    anchor: Option<InsertionAnchor>,
) {
    let Some(parent_id) = parent else {
        return;
    };
    ctx.use_helper("setInsertionState");
    let line = match anchor {
        Some(InsertionAnchor::Node(anchor_id)) => {
            cstr!("_setInsertionState(n{}, n{})", parent_id, anchor_id)
        }
        Some(InsertionAnchor::Index(index)) if index > 0 => {
            cstr!("_setInsertionState(n{}, {})", parent_id, index)
        }
        Some(InsertionAnchor::Index(_)) | None => cstr!("_setInsertionState(n{})", parent_id),
    };
    ctx.push_line(&line);
}
