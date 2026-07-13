//! Options API template-binding emission for the virtual TypeScript generator.

use vize_carton::{FxHashSet, String, append};
use vize_croquis::{BindingType, Croquis};

use super::options_api_support::{extend_options_api_descriptor_names, is_safe_value_identifier};
use crate::virtual_ts::props::OptionsApiPropsSource;
use crate::virtual_ts::types::VirtualTsOptions;

pub(super) fn generate_options_api_variables(
    mut ts: &mut String,
    summary: &Croquis,
    options: &VirtualTsOptions,
    script_facts: Option<&vize_atelier_sfc::SfcScriptGeneratorFacts>,
) {
    if summary.bindings.is_script_setup {
        return;
    }

    let macro_prop_names: FxHashSet<&str> = summary
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.as_str())
        .collect();
    let configured_globals: FxHashSet<&str> = options
        .template_globals
        .iter()
        .map(|global| global.name.as_str())
        .collect();
    let mut names: Vec<&str> = summary
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            let name = name.as_str();
            match binding_type {
                BindingType::Data | BindingType::Options | BindingType::VueGlobal => Some(name),
                BindingType::Props if !macro_prop_names.contains(name) => Some(name),
                _ => None,
            }
        })
        .filter(|name| !configured_globals.contains(name))
        .filter(|name| is_safe_value_identifier(name))
        .collect();
    names.sort_unstable();
    names.dedup();
    let inherited_unknown_names =
        unresolved_extends_template_names(summary, &configured_globals, script_facts);

    if names.is_empty() && inherited_unknown_names.is_empty() {
        return;
    }

    ts.push_str("  // Options API template bindings\n");
    ts.push_str(
        "  type __VizeOptionsInstance<T> = T extends abstract new (...args: any) => infer I ? I : any;\n",
    );
    ts.push_str(
        "  type __VizeOptionsBinding<T, K extends string> = K extends keyof __VizeOptionsInstance<T> ? __VizeOptionsInstance<T>[K] : any;\n",
    );
    for name in &names {
        append!(
            ts,
            "  const {name}: __VizeOptionsBinding<typeof __default__, \"{name}\"> = undefined as any;\n"
        );
    }
    if !inherited_unknown_names.is_empty() {
        ts.push_str("  // Unresolved imported Options API extends bindings\n");
        for name in &inherited_unknown_names {
            append!(ts, "  const {name}: any = undefined as any;\n");
        }
    }
    ts.push_str("  ");
    for name in &names {
        append!(ts, "void {name};");
    }
    for name in &inherited_unknown_names {
        append!(ts, "void {name};");
    }
    ts.push('\n');
}

fn unresolved_extends_template_names(
    summary: &Croquis,
    configured_globals: &FxHashSet<&str>,
    script_facts: Option<&vize_atelier_sfc::SfcScriptGeneratorFacts>,
) -> Vec<String> {
    if !script_facts.is_some_and(|facts| facts.has_unresolved_options_extends()) {
        return Vec::new();
    }

    let type_export_names: FxHashSet<&str> = summary
        .type_exports
        .iter()
        .map(|export| export.name.as_str())
        .collect();
    let used_components: FxHashSet<&str> = summary
        .used_components
        .iter()
        .map(|component| component.as_str())
        .collect();
    let mut names = summary
        .undefined_refs
        .iter()
        .filter_map(|reference| {
            let name = reference.name.as_str();
            if summary.bindings.bindings.contains_key(name)
                || configured_globals.contains(name)
                || type_export_names.contains(name)
                || used_components.contains(name)
                || !is_safe_value_identifier(name)
            {
                return None;
            }
            Some(String::from(name))
        })
        .collect::<Vec<_>>();
    for expression in &summary.template_expressions {
        collect_unresolved_extends_expression_names(
            &mut names,
            expression.content.as_str(),
            summary,
            configured_globals,
            &type_export_names,
            &used_components,
        );
        if let Some(guard) = expression.vif_guard.as_ref() {
            collect_unresolved_extends_expression_names(
                &mut names,
                guard.as_str(),
                summary,
                configured_globals,
                &type_export_names,
                &used_components,
            );
        }
    }
    names.sort();
    names.dedup();
    names
}

fn collect_unresolved_extends_expression_names(
    names: &mut Vec<String>,
    expression: &str,
    summary: &Croquis,
    configured_globals: &FxHashSet<&str>,
    type_export_names: &FxHashSet<&str>,
    used_components: &FxHashSet<&str>,
) {
    for identifier in vize_croquis::drawer::extract_identifiers_oxc(expression) {
        let name = identifier.as_str();
        if summary.bindings.bindings.contains_key(name)
            || configured_globals.contains(name)
            || type_export_names.contains(name)
            || used_components.contains(name)
            || !is_safe_value_identifier(name)
        {
            continue;
        }
        names.push(String::from(name));
    }
}

pub(super) fn generate_options_api_bridge(
    mut ts: &mut String,
    summary: &Croquis,
    script_facts: Option<&vize_atelier_sfc::SfcScriptGeneratorFacts>,
) {
    if summary.bindings.is_script_setup {
        return;
    }
    let Some(bridge) = script_facts.and_then(|facts| facts.options_api_bridge()) else {
        return;
    };
    if bridge.computed.is_empty() && bridge.methods.is_empty() {
        return;
    }

    let mut names: Vec<&str> = summary
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            let name = name.as_str();
            match binding_type {
                BindingType::Data | BindingType::Options | BindingType::Props => {
                    is_safe_value_identifier(name).then_some(name)
                }
                _ => None,
            }
        })
        .collect();
    extend_options_api_descriptor_names(&mut names, summary);
    names.sort_unstable();
    names.dedup();

    ts.push_str("  // Options API typed instance bridge\n");
    for (index, mapped_type) in bridge.mapped_types.iter().enumerate() {
        append!(
            ts,
            "  type __VizeOptionsMap{index} = {{ {mapped_type} }};\n"
        );
    }
    ts.push_str("  type __VizeThis = {\n");
    for name in names {
        append!(ts, "    {name}: any;\n");
    }
    ts.push_str("  }");
    for index in 0..bridge.mapped_types.len() {
        append!(ts, " & __VizeOptionsMap{index}");
    }
    ts.push_str(";\n");

    for function in &bridge.computed {
        emit_bridge_function(ts, "computed", function);
    }
    for function in &bridge.methods {
        emit_bridge_function(ts, "method", function);
    }

    ts.push_str("  ");
    let mut first = true;
    for function in bridge.computed.iter().chain(bridge.methods.iter()) {
        if !first {
            ts.push(' ');
        }
        append!(
            ts,
            "void __vize_{}_{};",
            script_options_function_kind_prefix(function.kind),
            function.safe_name
        );
        first = false;
    }
    ts.push_str("\n\n");
}

fn script_options_function_kind_prefix(
    kind: vize_atelier_sfc::ScriptOptionsFunctionKind,
) -> &'static str {
    match kind {
        vize_atelier_sfc::ScriptOptionsFunctionKind::Computed => "computed",
        vize_atelier_sfc::ScriptOptionsFunctionKind::Method => "method",
    }
}

fn emit_bridge_function(
    mut ts: &mut String,
    kind: &str,
    function: &vize_atelier_sfc::ScriptOptionsFunction,
) {
    let params = if function.params.is_empty() {
        String::from("this: __VizeThis")
    } else {
        let mut params = String::from("this: __VizeThis, ");
        params.push_str(&function.params);
        params
    };
    append!(
        ts,
        "  function __vize_{kind}_{}({params}) ",
        function.safe_name
    );
    ts.push_str(&function.body);
    ts.push('\n');
}

pub(super) fn options_api_props_from_facts(
    facts: Option<&vize_atelier_sfc::SfcScriptGeneratorFacts>,
) -> Option<OptionsApiPropsSource> {
    match facts?.options_api_props()? {
        vize_atelier_sfc::ScriptOptionsApiPropsSource::Object(source) => {
            Some(OptionsApiPropsSource::Object(source.clone()))
        }
        vize_atelier_sfc::ScriptOptionsApiPropsSource::DeferredObject(source) => {
            Some(OptionsApiPropsSource::DeferredObject(source.clone()))
        }
        vize_atelier_sfc::ScriptOptionsApiPropsSource::Names(names) => {
            Some(OptionsApiPropsSource::Names(names.clone()))
        }
    }
}
