//! Binding names owned by a v-for value pattern, including synthetic match scopes.

use vize_l0::{CompactString, SmallVec, smallvec};

pub(super) fn value_bindings(alias: &str) -> SmallVec<[CompactString; 4]> {
    if is_pattern(alias) {
        crate::steps::v_slot::extract_slot_prop_names(alias)
            .into_iter()
            .collect()
    } else {
        smallvec![CompactString::new(alias)]
    }
}

/// The names a destructured key or index alias binds beyond the single name
/// its scope slot records. A plain alias binds nothing more, and its pattern
/// is parsed only when it is one.
pub(super) fn pattern_tail_bindings(
    alias: Option<&str>,
    names: Option<&[&str]>,
) -> SmallVec<[CompactString; 4]> {
    match (names, alias) {
        (Some(names), _) => names
            .iter()
            .skip(1)
            .map(|name| CompactString::new(name))
            .collect(),
        (None, Some(alias)) if is_pattern(alias) => {
            crate::steps::v_slot::extract_slot_prop_names(alias)
                .into_iter()
                .collect()
        }
        _ => SmallVec::new(),
    }
}

fn is_pattern(alias: &str) -> bool {
    alias.trim_start().starts_with(['{', '['])
}
