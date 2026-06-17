use std::path::Path;

use vize_atelier_sfc::{SfcDescriptor, script::ScriptCompileContext};
use vize_carton::{FxHashSet, String as CompactString};

pub(super) fn augment_type_based_props_from_script_context(
    croquis: &mut vize_croquis::Croquis,
    descriptor: &SfcDescriptor<'_>,
    path: &Path,
) {
    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        return;
    };
    if croquis
        .macros
        .define_props()
        .is_none_or(|call| call.type_args.is_none())
    {
        return;
    }

    let mut ctx = ScriptCompileContext::new(&script_setup.content);
    let path_string = path.to_string_lossy();

    if let Some(script) = descriptor.script.as_ref()
        && !script.content.is_empty()
    {
        ctx.collect_types_from(&script.content);
        ctx.collect_imported_types_from_path(
            &script.content,
            path_string.as_ref(),
            matches!(script.lang.as_deref(), Some("ts" | "tsx")),
        );
    }
    ctx.collect_imported_types_from_path(
        &script_setup.content,
        path_string.as_ref(),
        matches!(script_setup.lang.as_deref(), Some("ts" | "tsx")),
    );
    ctx.analyze();

    let known_props = known_type_based_prop_names(croquis);
    let mut missing_props: Vec<CompactString> = ctx
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            matches!(binding_type, vize_relief::BindingType::Props)
                .then(|| name)
                .filter(|name| !known_props.contains(*name))
                .cloned()
        })
        .collect();
    if missing_props.is_empty() {
        return;
    }
    missing_props.sort();

    for name in missing_props {
        croquis
            .bindings
            .bindings
            .entry(name.clone())
            .or_insert(vize_relief::BindingType::Props);
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

fn known_type_based_prop_names(croquis: &vize_croquis::Croquis) -> FxHashSet<CompactString> {
    let mut names: FxHashSet<CompactString> = croquis
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

    // Resolve the named type's fields through the croquis TypeResolver, which
    // script analysis populates from the OXC AST, including local interfaces
    // and type literals.
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
    match trimmed.find('<') {
        Some(pos) => trimmed[..pos].trim_end(),
        None => trimmed,
    }
}
