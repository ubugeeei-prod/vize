//! Presence checks for directives without a value use the existing collected table.
use super::{DirectiveValueBindings, emit_unknown_directive_presence};
use crate::virtual_ts::VizeMapping;
use vize_carton::String;

pub(crate) fn generate_valueless_directive_presence(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    bindings: &DirectiveValueBindings,
    offset: u32,
    enabled: bool,
) {
    if !enabled {
        return;
    }
    let mut bindings: Vec<_> = bindings
        .values()
        .filter(|binding| !binding.has_value)
        .collect();
    bindings.sort_unstable_by_key(|binding| binding.name_range);
    for binding in bindings {
        emit_unknown_directive_presence(ts, mappings, binding, offset, "  ", true);
    }
}
