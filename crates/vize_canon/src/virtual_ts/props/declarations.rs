//! Module-level Props type declaration emission.

use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, macros::ModelDefinition};

use super::{
    PropsTypeEmission, add_generic_defaults, setup_scoped::unused_generic_comment,
    strip_const_modifiers,
};

/// Runtime `props:` source for Options API `export type Props` emission.
/// `DeferredObject` is routed through setup scope for value-only object syntax.
pub(crate) enum OptionsApiPropsSource {
    Object(String),
    DeferredObject(String),
    Names(Vec<String>),
}

fn model_prop_type(model: &ModelDefinition) -> &str {
    model.model_type.as_deref().unwrap_or("unknown")
}

fn emit_model_prop_member(ts: &mut String, model: &ModelDefinition) {
    let optional = if model.required { "" } else { "?" };
    let name = model.name.as_str();
    let prop_type = model_prop_type(model);
    append!(*ts, "  \"{name}\"{optional}: {prop_type};\n");
}

pub(crate) fn append_model_props_type_literal(ts: &mut String, models: &[ModelDefinition]) {
    ts.push_str("{\n");
    for model in models {
        emit_model_prop_member(ts, model);
    }
    ts.push('}');
}

pub(crate) fn generate_props_type(
    ts: &mut String,
    summary: &Croquis,
    generic_param: Option<&str>,
    options_api_props: Option<&OptionsApiPropsSource>,
    emission: PropsTypeEmission,
    define_props_type_references: Option<&FxHashSet<String>>,
) {
    let props = summary.macros.props();
    let has_props = !props.is_empty();
    let models = summary.macros.models();
    let has_models = !models.is_empty();
    let define_props_type_args = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref());
    let props_already_defined = summary
        .type_exports
        .iter()
        .any(|te| te.name.as_str() == "Props");
    let generic_decl = generic_param
        .map(|generic| {
            let with_defaults = strip_const_modifiers(&add_generic_defaults(generic));
            cstr!("<{with_defaults}>")
        })
        .unwrap_or_default();

    ts.push_str("// ========== Exported Types ==========\n");
    if props_already_defined
        || emission == PropsTypeEmission::DeferredToSetup && define_props_type_args.is_some()
    {
    } else if let Some(type_args) = define_props_type_args {
        let inner_type = type_args
            .strip_prefix('<')
            .and_then(|source| source.strip_suffix('>'))
            .unwrap_or(type_args.as_str());
        ts.push_str(unused_generic_comment(
            generic_param,
            define_props_type_references,
        ));
        if has_models {
            append!(*ts, "export type Props{generic_decl} = {inner_type} & ");
            append_model_props_type_literal(ts, models);
            ts.push_str(";\n");
        } else {
            append!(*ts, "export type Props{generic_decl} = {inner_type};\n");
        }
    } else if has_props || has_models {
        append!(*ts, "export type Props{generic_decl} = {{\n");
        let mut emitted_names: FxHashSet<String> = FxHashSet::default();
        for prop in props {
            let prop_type = prop.prop_type.as_deref().unwrap_or("unknown");
            let optional = if prop.required { "" } else { "?" };
            append!(*ts, "  {}{optional}: {prop_type};\n", prop.name);
            emitted_names.insert(prop.name.as_str().into());
        }
        for model in models {
            if !emitted_names.contains(model.name.as_str()) {
                emit_model_prop_member(ts, model);
            }
        }
        ts.push_str("};\n");
    } else if let Some(options_api_props) = options_api_props {
        emit_options_api_props_type(ts, &generic_decl, options_api_props);
    } else {
        append!(*ts, "export type Props{generic_decl} = {{}};\n");
    }
    ts.push('\n');
}

fn emit_options_api_props_type(
    ts: &mut String,
    generic_decl: &str,
    options_api_props: &OptionsApiPropsSource,
) {
    match options_api_props {
        OptionsApiPropsSource::Object(source) => {
            append!(
                *ts,
                "export type Props{generic_decl} = __RuntimePropShape<{source}>;\n"
            );
        }
        OptionsApiPropsSource::DeferredObject(_) => {}
        OptionsApiPropsSource::Names(names) => {
            append!(*ts, "export type Props{generic_decl} = {{\n");
            for name in names {
                append!(*ts, "  \"{name}\"?: unknown;\n");
            }
            ts.push_str("};\n");
        }
    }
}
