use vize_carton::{FxHashSet, String};
use vize_croquis::Croquis;

use super::super::helpers::is_reserved_identifier;
use super::template_bindings::should_skip_template_prop_binding;
use super::{strip_outer_angle_brackets, type_reference_lookup_key};

pub(crate) fn collect_template_prop_names(summary: &Croquis) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    let props = summary.macros.props();
    let type_properties = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref())
        .map(|type_args| {
            let type_name = strip_outer_angle_brackets(type_args.trim());
            summary
                .types
                .extract_properties(type_reference_lookup_key(type_name))
        })
        .unwrap_or_default();

    if !type_properties.is_empty() {
        for prop in &type_properties {
            insert_reserved_prop_name(summary, &mut names, prop.name.as_str());
        }
    } else if !props.is_empty() {
        for prop in props {
            insert_reserved_prop_name(summary, &mut names, prop.name.as_str());
        }
        return names;
    }

    for model in summary.macros.models() {
        insert_reserved_prop_name(summary, &mut names, model.name.as_str());
    }

    names
}

fn insert_reserved_prop_name(summary: &Croquis, names: &mut FxHashSet<String>, name: &str) {
    if should_skip_template_prop_binding(summary, name) || !is_reserved_identifier(name) {
        return;
    }
    names.insert(name.into());
}
