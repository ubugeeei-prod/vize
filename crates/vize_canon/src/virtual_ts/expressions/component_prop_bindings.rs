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
        .filter(|prop| is_checkable_prop(prop) && prop.name.as_str() == name)
        .filter_map(|prop| {
            generated_prop_value(prop, template_binding_access).map(|value| (prop, value))
        })
        .collect()
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
            let mut value = String::default();
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
            Some(value)
        }
    }
}
