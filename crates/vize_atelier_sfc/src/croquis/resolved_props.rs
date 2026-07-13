use vize_carton::{FxHashSet, String};
use vize_croquis::Croquis;

use crate::compile::is_ts_lang;
use crate::script::ScriptCompileContext;
use crate::types::{BindingType, SfcDescriptor};

/// Merge props resolved by the script compile context into a Croquis summary.
///
/// This is the single cross-file/node_modules resolution boundary shared by
/// compiler, type checker, and Atlas SFC products. Both binding identity and
/// macro prop definitions are completed before template analysis.
pub fn merge_resolved_props_into_croquis(
    croquis: &mut Croquis,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
) {
    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        return;
    };
    let mut context = ScriptCompileContext::new(&script_setup.content);
    if let Some(script) = descriptor.script.as_ref() {
        context.collect_types_from(&script.content);
    }
    if !filename.is_empty() {
        context.collect_imported_types_from_path(
            &script_setup.content,
            filename,
            is_ts_lang(script_setup.lang.as_deref()),
        );
        if let Some(script) = descriptor.script.as_ref() {
            context.collect_imported_types_from_path(
                &script.content,
                filename,
                is_ts_lang(script.lang.as_deref()),
            );
        }
    }
    context.analyze();

    let known_props = known_type_based_prop_names(croquis);
    let top_level_props = croquis
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_ref())
        .map(|type_args| context.resolve_type_prop_names(strip_outer_angle_brackets(type_args)));
    let mut resolved_props: Vec<_> = context
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            matches!(binding_type, BindingType::Props | BindingType::PropsAliased)
                .then_some(name)
                .filter(|name| {
                    top_level_props
                        .as_ref()
                        .is_some_and(|props| props.contains(*name))
                })
                .cloned()
        })
        .collect();
    resolved_props.sort();
    resolved_props.dedup();
    for name in resolved_props {
        croquis
            .bindings
            .bindings
            .entry(name.clone())
            .or_insert(BindingType::Props.into());
        if !known_props.contains(name.as_str()) {
            croquis
                .macros
                .add_prop(vize_croquis::macros::PropDefinition {
                    name,
                    prop_type: None,
                    required: false,
                    default_value: None,
                });
        }
    }
}

fn known_type_based_prop_names(croquis: &Croquis) -> FxHashSet<String> {
    let mut names: FxHashSet<_> = croquis
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.clone())
        .collect();
    let Some(type_args) = croquis
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_ref())
    else {
        return names;
    };
    let type_name = strip_outer_angle_brackets(type_args.trim());
    for prop in croquis
        .types
        .extract_properties(type_reference_lookup_key(type_name))
    {
        names.insert(prop.name);
    }
    names
}

fn strip_outer_angle_brackets(value: &str) -> &str {
    value
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(value)
}

fn type_reference_lookup_key(type_name: &str) -> &str {
    let trimmed = type_name.trim();
    if trimmed.starts_with('{') {
        return type_name;
    }
    trimmed
        .find('<')
        .map_or(trimmed, |position| trimmed[..position].trim_end())
}
