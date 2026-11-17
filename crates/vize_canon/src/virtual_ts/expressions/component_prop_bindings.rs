//! Shared extraction and merging for generated component class/style bindings.

use super::component_props::is_checkable_prop;
use super::prop_sources::{append_prop_value, generated_prop_value};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::String;
use vize_croquis::croquis::{ComponentUsage, PassedProp};

pub(super) fn collect_generated_class_bindings<'a>(
    usage: &'a ComponentUsage,
    template_binding_access: &TemplateBindingAccess,
) -> Vec<(&'a PassedProp, String)> {
    collect_generated_named_bindings(usage, template_binding_access, "class")
}

pub(super) fn collect_generated_named_bindings<'a>(
    usage: &'a ComponentUsage,
    template_binding_access: &TemplateBindingAccess,
    name: &str,
) -> Vec<(&'a PassedProp, String)> {
    usage
        .props
        .iter()
        .filter(|prop| {
            is_checkable_prop(prop)
                && prop.name.as_str() == name
                && !overridden_by_later_static(usage, prop)
        })
        .filter_map(|prop| {
            generated_prop_value(prop, template_binding_access).map(|value| (prop, value))
        })
        .collect()
}

/// Whether a later static attribute of the same name replaces this dynamic
/// `class`/`style` binding as the value the child's props are checked with.
///
/// Native elements follow the same rule (`overridden_by_later_attribute` in
/// `native_props.rs`): vue-tsc writes an element's attributes into one object
/// literal in authored order, so `<Comp :class="x" class="row">` checks the
/// static `"row"` and never compares `x` with the child's `class` prop, while
/// `<Comp class="row" :class="x">` does. Vue merges both forms through
/// `normalizeClass`/`normalizeStyle`, which take any value, so the child never
/// receives `x` itself. The expression is still checked on its own.
pub(crate) fn overridden_by_later_static(usage: &ComponentUsage, prop: &PassedProp) -> bool {
    prop.is_dynamic
        && !prop.name_is_dynamic
        && matches!(prop.name.as_str(), "class" | "style")
        && usage.props.iter().any(|later| {
            later.start > prop.start
                && !later.is_dynamic
                && !later.name_is_dynamic
                && later.name == prop.name
        })
}

pub(super) fn merged_class_binding_value(
    bindings: &[(&PassedProp, String)],
    mut ranges: Option<&mut Vec<std::ops::Range<usize>>>,
) -> Option<String> {
    match bindings {
        [] => None,
        [(_, value)] => {
            if let Some(ranges) = ranges.as_mut() {
                ranges.push(0..value.len());
            }
            Some(value.clone())
        }
        _ => {
            let mut value = crate::virtual_ts::TextBuf::default();
            value.push('[');
            for (index, (_, binding_value)) in bindings.iter().enumerate() {
                if index > 0 {
                    value.push_str(", ");
                }
                let generated = append_prop_value(&mut value, binding_value.as_str());
                if let Some(ranges) = ranges.as_mut() {
                    ranges.push(generated);
                }
            }
            value.push(']');
            Some(value.into_compact())
        }
    }
}
