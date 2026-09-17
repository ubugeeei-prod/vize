//! Binding names owned by a v-for value pattern, including synthetic match scopes.

use vize_s0::{CompactString, SmallVec, smallvec};

pub(super) fn value_bindings(alias: &str) -> SmallVec<[CompactString; 4]> {
    if alias.trim_start().starts_with(['{', '[']) {
        crate::steps::v_slot::extract_slot_prop_names(alias)
            .into_iter()
            .collect()
    } else {
        smallvec![CompactString::new(alias)]
    }
}
