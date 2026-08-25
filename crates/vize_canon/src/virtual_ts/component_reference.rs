use vize_carton::{CompactString, FxHashSet, String, camelize, capitalize, cstr};
use vize_croquis::{BindingType, Croquis};

use super::{
    helpers::{to_safe_identifier, to_safe_identifier_fragment},
    types::VirtualTsOptions,
};

pub(crate) fn component_reference_alias(template_name: &str) -> String {
    cstr!(
        "__VizeComponent_{}",
        to_safe_identifier_fragment(template_name).as_str()
    )
}

pub(crate) fn component_binding_reference(
    summary: &Croquis,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    template_name: &str,
) -> String {
    resolved_component_binding_reference(
        summary,
        options,
        syntactic_type_only_imported_names,
        template_name,
    )
    .unwrap_or_else(|| to_safe_identifier(template_name))
}

pub(crate) fn resolved_component_binding_reference(
    summary: &Croquis,
    options: &VirtualTsOptions,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    template_name: &str,
) -> Option<String> {
    let camel_name = camelize(template_name);
    let pascal_name = capitalize(camel_name.as_str());
    // Value bindings win over a type-only PascalCase collision
    // (`chartComponent` vs `import type { ChartComponent }`).
    for candidate in [template_name, camel_name.as_str(), pascal_name.as_str()] {
        if let Some(binding_type) = summary.bindings.get(candidate)
            && !contains_compact_name(syntactic_type_only_imported_names, candidate)
        {
            return Some(component_binding_reference_for_summary_binding(
                summary,
                candidate,
                binding_type,
            ));
        }
        if options
            .external_template_bindings
            .iter()
            .any(|name| name.as_str() == candidate)
        {
            return Some(String::from(candidate));
        }
    }
    if has_type_only_component_candidate(syntactic_type_only_imported_names, template_name) {
        return Some(component_reference_alias(template_name));
    }
    None
}

fn component_binding_reference_for_summary_binding(
    summary: &Croquis,
    candidate: &str,
    binding_type: BindingType,
) -> String {
    if matches!(binding_type, BindingType::Props | BindingType::PropsAliased)
        && !is_props_destructure_local(summary, candidate)
    {
        return to_safe_identifier(candidate);
    }
    String::from(candidate)
}

fn is_props_destructure_local(summary: &Croquis, candidate: &str) -> bool {
    summary
        .macros
        .props_destructure()
        .is_some_and(|destructure| {
            destructure
                .bindings
                .values()
                .any(|binding| binding.local.as_str() == candidate)
                || destructure
                    .rest_id
                    .as_ref()
                    .is_some_and(|rest_id| rest_id.as_str() == candidate)
        })
}

pub(crate) fn has_type_only_component_candidate(
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
    template_name: &str,
) -> bool {
    let camel_name = camelize(template_name);
    let pascal_name = capitalize(camel_name.as_str());
    [template_name, camel_name.as_str(), pascal_name.as_str()]
        .iter()
        .any(|candidate| contains_compact_name(syntactic_type_only_imported_names, candidate))
}

pub(crate) fn contains_compact_name(names: &FxHashSet<CompactString>, name: &str) -> bool {
    names.iter().any(|candidate| candidate.as_str() == name)
}
