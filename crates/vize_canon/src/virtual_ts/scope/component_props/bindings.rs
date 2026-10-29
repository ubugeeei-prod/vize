//! One prepared script-binding view for a component-usage lookup domain.
use super::super::context::{ComponentBindingCheck, ComponentPropsContext};
use super::named_dynamic_reference;
use crate::virtual_ts::script_facts::{self, ScriptBindings};
use vize_carton::{FxHashSet, camelize, capitalize};
use vize_croquis::analysis::ComponentUsage;

pub(in crate::virtual_ts::scope) fn collect_checkable_usages(
    ctx: &ComponentPropsContext<'_, '_>,
) -> Vec<(usize, ComponentUsage)> {
    let external_template_bindings: FxHashSet<&str> = ctx
        .options
        .external_template_bindings
        .iter()
        .map(|name| name.as_str())
        .collect();
    let usages = vize_croquis::facts::component_usage_list(ctx.summary);
    if usages.is_empty() {
        return Vec::new();
    }
    script_facts::with_bindings(ctx.summary, |bindings| {
        usages
            .into_iter()
            .enumerate()
            .filter(|(_, usage)| {
                component_usage_has_checkable_binding(
                    bindings,
                    usage,
                    &external_template_bindings,
                    ctx.component_binding_check,
                    ctx.legacy_vue2,
                ) || named_dynamic_reference(ctx, usage).is_some()
            })
            .collect()
    })
}

pub(in crate::virtual_ts::scope) fn component_usage_has_checkable_binding(
    bindings: ScriptBindings<'_>,
    usage: &ComponentUsage,
    external_template_bindings: &FxHashSet<&str>,
    component_binding_check: ComponentBindingCheck<'_>,
    legacy_vue2: bool,
) -> bool {
    let name = usage.name.as_str();
    let script_binding = if bindings.contains_binding(name) {
        true
    } else {
        let camel = camelize(name);
        let pascal = capitalize(camel.as_str());
        [pascal.as_str(), camel.as_str()]
            .into_iter()
            .any(|candidate| bindings.contains_binding(candidate))
    };
    script_binding
        || (!legacy_vue2
            && (component_name_matches_external_template_binding(name, external_template_bindings)
                || component_binding_check.allows(name)
                || vize_croquis::drawer::is_dynamic_component_alias(name)))
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
