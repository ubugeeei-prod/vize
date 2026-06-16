//! Patch flag calculation and naming functions.

use super::patch_flag_inner::calculate_element_patch_info_inner;
use crate::ElementNode;
use crate::options::BindingMetadata;
use vize_carton::String;
use vize_carton::ToCompactString;

/// Calculate patch flag and dynamic props for an element.
/// `skip_is_prop`: when true, skip `:is` binding (used for `<component :is="...">`)
pub fn calculate_element_patch_info(
    el: &ElementNode<'_>,
    bindings: Option<&BindingMetadata>,
    cache_handlers: bool,
) -> (Option<i32>, Option<Vec<String>>) {
    calculate_element_patch_info_inner(el, bindings, cache_handlers, false)
}

/// Same as `calculate_element_patch_info` but allows skipping the `is` prop.
pub fn calculate_element_patch_info_skip_is(
    el: &ElementNode<'_>,
    bindings: Option<&BindingMetadata>,
    cache_handlers: bool,
) -> (Option<i32>, Option<Vec<String>>) {
    calculate_element_patch_info_inner(el, bindings, cache_handlers, true)
}

/// Get patch flag name for comment
pub fn patch_flag_name(flag: i32) -> String {
    // Single flag matches
    match flag {
        1 => return "TEXT".to_compact_string(),
        2 => return "CLASS".to_compact_string(),
        4 => return "STYLE".to_compact_string(),
        8 => return "PROPS".to_compact_string(),
        16 => return "FULL_PROPS".to_compact_string(),
        32 => return "NEED_HYDRATION".to_compact_string(),
        64 => return "STABLE_FRAGMENT".to_compact_string(),
        128 => return "KEYED_FRAGMENT".to_compact_string(),
        256 => return "UNKEYED_FRAGMENT".to_compact_string(),
        512 => return "NEED_PATCH".to_compact_string(),
        1024 => return "DYNAMIC_SLOTS".to_compact_string(),
        _ => {}
    }

    // Multiple flags - build combined string
    let mut names = Vec::new();
    if flag & 1 != 0 {
        names.push("TEXT");
    }
    if flag & 2 != 0 {
        names.push("CLASS");
    }
    if flag & 4 != 0 {
        names.push("STYLE");
    }
    if flag & 8 != 0 {
        names.push("PROPS");
    }
    if flag & 16 != 0 {
        names.push("FULL_PROPS");
    }
    if flag & 32 != 0 {
        names.push("NEED_HYDRATION");
    }
    if flag & 64 != 0 {
        names.push("STABLE_FRAGMENT");
    }
    if flag & 128 != 0 {
        names.push("KEYED_FRAGMENT");
    }
    if flag & 256 != 0 {
        names.push("UNKEYED_FRAGMENT");
    }
    if flag & 512 != 0 {
        names.push("NEED_PATCH");
    }
    if flag & 1024 != 0 {
        names.push("DYNAMIC_SLOTS");
    }
    if flag & 2048 != 0 {
        names.push("DEV_ROOT_FRAGMENT");
    }

    if names.is_empty() {
        "UNKNOWN".to_compact_string()
    } else {
        names.join(", ").into()
    }
}
