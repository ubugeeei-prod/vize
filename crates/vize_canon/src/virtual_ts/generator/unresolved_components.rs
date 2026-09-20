//! Template component value declarations that preserve real imported values.

use vize_carton::{CompactString, FxHashSet, String, append, camelize, capitalize};
use vize_croquis::Croquis;

use super::global_components::GlobalComponentPlan;
use crate::virtual_ts::{
    component_reference::{
        component_binding_reference, component_reference_alias, contains_compact_name,
        has_type_only_component_candidate,
    },
    helpers::to_safe_identifier,
    types::VirtualTsOptions,
};

pub(super) fn emit_unresolved_components(
    ts: &mut String,
    summary: &Croquis,
    options: &VirtualTsOptions,
    global_components: &GlobalComponentPlan,
    syntactic_type_only_imported_names: &FxHashSet<CompactString>,
) {
    if summary.used_components.is_empty() {
        return;
    }

    let external_template_bindings: FxHashSet<&str> = options
        .external_template_bindings
        .iter()
        .map(|name| name.as_str())
        .collect();
    let mut has_unresolved = false;
    for component in &summary.used_components {
        let name = component.as_str();
        // A dynamic `:is` alias is declared by the template scope itself.
        if vize_croquis::drawer::is_dynamic_component_alias(name)
            || (summary.bindings.bindings.contains_key(name)
                && !contains_compact_name(syntactic_type_only_imported_names, name))
            || component_name_matches_external_template_binding(name, &external_template_bindings)
            || global_components.keeps_unresolved_binding(name)
        {
            continue;
        }
        if !has_unresolved {
            ts.push_str("\n  // Auto-imported/built-in components (not in script bindings)\n");
            has_unresolved = true;
        }
        let safe = if has_type_only_component_candidate(syntactic_type_only_imported_names, name) {
            component_reference_alias(name)
        } else {
            to_safe_identifier(name)
        };
        // An unresolved name is an error type, not a declared `any`: both are
        // unchecked, but TypeScript stays silent about values derived from an
        // error type where a declared `any` still fails a `never` parameter.
        // `vue-tsc` resolves the tag through a missing registry key, so its
        // slot payloads and listener arguments are silent in the same way.
        append!(
            *ts,
            "  // @ts-ignore Unresolved component: an error type, never reported.\n  const {safe}: {{}}[\"{name}\"] = undefined as any;\n"
        );
    }

    ts.push_str("\n  // Mark used components as referenced\n");
    for component in &summary.used_components {
        if vize_croquis::drawer::is_dynamic_component_alias(component.as_str()) {
            continue;
        }
        let safe = component_binding_reference(
            summary,
            options,
            syntactic_type_only_imported_names,
            component.as_str(),
        );
        append!(*ts, "  void {safe};\n");
    }
}

fn component_name_matches_external_template_binding(
    name: &str,
    external_template_bindings: &FxHashSet<&str>,
) -> bool {
    let camel_name = camelize(name);
    let pascal_name = capitalize(camel_name.as_str());
    [name, camel_name.as_str(), pascal_name.as_str()]
        .iter()
        .any(|candidate| external_template_bindings.contains(candidate))
}
