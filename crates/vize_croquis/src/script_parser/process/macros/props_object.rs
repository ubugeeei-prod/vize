//! The binding a `defineProps` or `withDefaults` call initializes.

use crate::{reactivity::ReactiveKind, script_parser::ScriptParseResult};
use vize_carton::CompactString;

/// Register `name` as the component's props object: a readonly reactive
/// source, and the binding fact consumers use to tell the props object apart
/// from an unrelated authored binding that happens to share a name.
pub(super) fn register(result: &mut ScriptParseResult, name: &str, at: u32) {
    result
        .reactivity
        .register(CompactString::new(name), ReactiveKind::Readonly, at);
    result
        .macros
        .set_props_object_local(CompactString::new(name));
}
