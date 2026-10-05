//! Local binding replacement ends its previous plain-snapshot provenance.

use super::ScriptParseResult;

pub(super) fn clear_replaced_origin(result: &mut ScriptParseResult, target: &str) {
    if !result.reactive_value_origins.contains_key(target) {
        return;
    }
    // A same-named parameter/local does not replace the recorded outer binding.
    let shadowed = result.binding_spans.get(target).is_some_and(|&(start, _)| {
        result
            .scopes
            .lookup(target)
            .is_some_and(|(_, binding)| binding.declaration_offset != start)
    });
    if !shadowed {
        result.reactive_value_origins.remove(target);
    }
}
