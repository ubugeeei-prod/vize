//! Which named entries of a props literal leave their value to the per-prop
//! assignment. The literal and the content-mapper per-prop emitter read the
//! same rows, so a value is owned by exactly one of them.

use super::super::component_props::is_checkable_prop;
use crate::virtual_ts::helpers::{component_aria_prop_keys, component_prop_key};
use vize_carton::FxHashSet;
use vize_croquis::croquis::ComponentUsage;

/// One row per prop of `usage`: `None` when the literal writes no named
/// entry, `Some(true)` for an `@ts-ignore`d entry whose value the per-prop
/// assignment owns (an ARIA pair, or a key a later `v-bind` spread
/// overwrites, once per key), `Some(false)` for an ordinary entry. The merge
/// flags only change `class` and `style` rows.
pub(crate) fn named_entry_marks(
    usage: &ComponentUsage,
    (merge_class, merge_style): (bool, bool),
) -> Vec<Option<bool>> {
    let mut keys = FxHashSet::default();
    let (mut class_written, mut style_written) = (false, false);
    usage
        .props
        .iter()
        .map(|prop| {
            if !is_checkable_prop(prop) {
                return None;
            }
            let merged = match prop.name.as_str() {
                "class" if merge_class => Some(&mut class_written),
                "style" if merge_style => Some(&mut style_written),
                _ => None,
            };
            if let Some(written) = merged {
                if *written {
                    return None;
                }
                *written = true;
            } else if prop.is_dynamic && prop.value.is_none() {
                return None;
            }
            let aria = component_aria_prop_keys(prop.name.as_str());
            let key = aria.as_ref().map_or_else(
                || component_prop_key(prop.name.as_str()),
                |(_, camel)| camel.clone(),
            );
            let repeated = !keys.insert(key);
            let before_spread = usage
                .spread_props
                .iter()
                .any(|spread| spread.start >= prop.start);
            Some(aria.is_some() || (before_spread && !repeated))
        })
        .collect()
}

/// Whether the literal's entry for `usage.props[index]` checks the authored
/// value against the child's typed inputs: an unmarked entry that is neither
/// an inline callback (written as `never`) nor Vue's always-public `class` or
/// `style`.
pub(crate) fn literal_carries_value(
    usage: &ComponentUsage,
    marks: &[Option<bool>],
    index: usize,
) -> bool {
    marks.get(index) == Some(&Some(false))
        && usage.props.get(index).is_some_and(|prop| {
            !crate::virtual_ts::scope::is_inline_callback_prop(prop)
                && !matches!(prop.name.as_str(), "class" | "style")
        })
}
