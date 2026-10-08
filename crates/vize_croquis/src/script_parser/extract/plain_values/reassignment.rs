//! Local binding replacement ends its previous plain-snapshot provenance.

use super::ScriptParseResult;

pub(super) fn clear_replaced_origin(result: &mut ScriptParseResult, target: &str) {
    result.clear_reactive_origin(target);
}
