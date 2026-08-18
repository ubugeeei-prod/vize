use vize_carton::{CompactString, FxHashSet, String, camelize, capitalize, cstr};
use vize_croquis::Croquis;

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
    if has_type_only_component_candidate(syntactic_type_only_imported_names, template_name) {
        return component_reference_alias(template_name);
    }

    let camel_name = camelize(template_name);
    let pascal_name = capitalize(camel_name.as_str());
    for candidate in [template_name, camel_name.as_str(), pascal_name.as_str()] {
        if summary.bindings.bindings.contains_key(candidate)
            || options
                .external_template_bindings
                .iter()
                .any(|name| name.as_str() == candidate)
        {
            return to_safe_identifier(candidate);
        }
    }
    to_safe_identifier(template_name)
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
