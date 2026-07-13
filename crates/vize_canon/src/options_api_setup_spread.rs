use vize_carton::{FxHashSet, String};
use vize_croquis::{BindingType, Croquis, OptionGroup};

pub(crate) fn suppresses_template_undefined_refs(
    options_api_enabled: bool,
    script_facts: Option<&vize_atelier_sfc::SfcScriptGeneratorFacts>,
) -> bool {
    options_api_enabled && script_facts.is_some_and(|facts| facts.options_setup_return_has_spread())
}

pub(crate) fn collect_template_setup_bindings(
    summary: &Croquis,
    options_api: bool,
    template_referenced_names: Option<&FxHashSet<String>>,
    script_facts: Option<&vize_atelier_sfc::SfcScriptGeneratorFacts>,
) -> Vec<String> {
    let mut names = collect_descriptor_setup_bindings(summary, options_api);
    if suppresses_template_undefined_refs(options_api, script_facts) {
        extend_spread_bindings(&mut names, summary, template_referenced_names);
    }
    if let Some(template_referenced_names) = template_referenced_names {
        names.retain(|name| template_referenced_names.contains(name.as_str()));
    }
    names.sort_unstable();
    names.dedup();
    names
}

fn collect_descriptor_setup_bindings(summary: &Croquis, options_api: bool) -> Vec<String> {
    if !options_api || summary.bindings.is_script_setup {
        return Vec::new();
    }
    let Some(descriptor) = summary.options_descriptor.as_ref() else {
        return Vec::new();
    };
    descriptor
        .members_in(OptionGroup::Setup)
        .map(|member| member.name.as_str())
        .filter(|name| {
            is_safe_value_identifier(name)
                && matches!(
                    summary.bindings.get(name),
                    Some(BindingType::SetupMaybeRef | BindingType::SetupRef)
                )
        })
        .map(String::from)
        .collect()
}

fn extend_spread_bindings(
    names: &mut Vec<String>,
    summary: &Croquis,
    template_referenced_names: Option<&FxHashSet<String>>,
) {
    if let Some(template_referenced_names) = template_referenced_names {
        names.extend(
            template_referenced_names
                .iter()
                .filter(|name| is_safe_spread_binding(summary, name.as_str()))
                .map(|name| String::from(name.as_str())),
        );
        return;
    }
    names.extend(
        summary
            .undefined_refs
            .iter()
            .filter(|reference| reference.context == "template expression")
            .map(|reference| reference.name.as_str())
            .filter(|name| is_safe_spread_binding(summary, name))
            .map(String::from),
    );
}

fn is_safe_spread_binding(summary: &Croquis, name: &str) -> bool {
    is_safe_value_identifier(name) && summary.bindings.get(name).is_none()
}

fn is_safe_value_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}
