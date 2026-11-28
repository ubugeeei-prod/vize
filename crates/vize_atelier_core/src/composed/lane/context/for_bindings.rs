// DO NOT EDIT. Generated from crates/vize_atelier_core/src/lane/context/for_bindings.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
//! Binding names owned by a v-for value pattern, including synthetic match scopes.

use vize_l0::{CompactString, SmallVec, smallvec};

pub(super) fn value_bindings(alias: &str) -> SmallVec<[CompactString; 4]> {
    if is_pattern(alias) {
        crate::composed::steps::v_slot::extract_slot_prop_names(alias)
            .into_iter()
            .collect()
    } else {
        smallvec![CompactString::new(alias)]
    }
}

/// Whether a v-for alias is a destructuring pattern rather than one name.
#[inline]
pub(super) fn is_pattern(alias: &str) -> bool {
    alias.trim_start().starts_with(['{', '['])
}
