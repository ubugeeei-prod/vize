use oxc_syntax::identifier::is_identifier_name;
use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::{
    Croquis,
    macros::{ModelDefinition, PropDefinition},
};
use vize_l0::ts;

use crate::virtual_ts::{
    helpers::push_ts_string_literal, model_types::model_value_type, types::RESOLVED_RUNTIME_DEFAULT,
};

use crate::virtual_ts::generic_parameters::GenericParameters;

use super::template_model_modifiers::{model_modifier_prop_name, model_modifier_type};

fn emit_type_member(
    ts: &mut crate::virtual_ts::TextBuf,
    name: &str,
    optional: &str,
    ty: &str,
    prefer_identifier: bool,
) {
    ts.push_str("  ");
    if prefer_identifier && is_identifier_name(name) {
        ts.push_str(name);
    } else {
        push_ts_string_literal(ts, name);
    }
    append!(*ts, "{optional}: {ty};\n");
}

fn emit_model_doc(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    script: Option<&str>,
    model: &ModelDefinition,
) {
    let Some(doc) = script.and_then(|script| {
        super::runtime_docs::model_doc(script, &summary.macros, model.name.as_str())
    }) else {
        return;
    };
    ts.push_str("  ");
    ts.push_str(doc);
    ts.push('\n');
}

/// Vue publishes a runtime prop that carries a `default` (even `undefined`) as
/// optional to the parent although it is `required: true` (`$props` is
/// `Partial<Defaults> & Omit<Props, keyof Defaults>`), except a plain `Boolean`
/// prop, whose `required: true` outranks its default. `vue-tsc` checks the
/// parent against that `$props`, so the published member must agree with it;
/// the setup-side `props` binding is typed through `__WithDefaultsResult` and
/// keeps the defaulted value either way.
///
/// A prop the runtime resolver collected from a spread or an import carries
/// only the [`RESOLVED_RUNTIME_DEFAULT`] placeholder, which stands for a
/// `Boolean` declaration as much as for a real default; the resolver has
/// already settled `required` for the parent, so the placeholder is not a
/// default here.
fn parent_requires_prop(prop: &PropDefinition) -> bool {
    if !prop.required {
        return false;
    }
    let has_default = prop
        .default_value
        .as_deref()
        .is_some_and(|default| default != RESOLVED_RUNTIME_DEFAULT);
    !has_default || prop.prop_type.as_deref() == Some("boolean")
}

fn emit_model_prop_members(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    script: Option<&str>,
    model: &ModelDefinition,
    generic_parameters: Option<&GenericParameters>,
    emitted_names: &mut FxHashSet<String>,
) {
    let optional = if model.required { "" } else { "?" };
    let name = model.name.as_str();
    let prop_type = model_value_type(generic_parameters, model);
    if emitted_names.insert(name.into()) {
        emit_model_doc(ts, summary, script, model);
        emit_type_member(ts, name, optional, prop_type.as_str(), false);
    }

    let modifiers_name = model_modifier_prop_name(name);
    let modifier_type = model_modifier_type(summary, model);
    if emitted_names.insert(modifiers_name.clone()) {
        let lib = crate::virtual_ts::generated_names::lib;
        let modifier_type = cstr!(
            "{}<{}<{modifier_type}, true>>",
            lib("Partial"),
            lib("Record")
        );
        emit_type_member(
            ts,
            modifiers_name.as_str(),
            "?",
            modifier_type.as_str(),
            false,
        );
    }
}

pub(crate) fn emitted_model_prop_names(
    summary: &Croquis,
    models: &[ModelDefinition],
) -> FxHashSet<String> {
    let mut occupied_names: FxHashSet<String> = summary
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.as_str().into())
        .collect();
    let mut emitted_models = FxHashSet::default();
    for model in models {
        if occupied_names.insert(model.name.as_str().into()) {
            emitted_models.insert(model.name.as_str().into());
        }
        occupied_names.insert(model_modifier_prop_name(model.name.as_str()));
    }
    emitted_models
}

pub(super) fn append_model_props_type_literal(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    script: Option<&str>,
    models: &[ModelDefinition],
    generic_parameters: Option<&GenericParameters>,
) {
    ts! { &mut *ts =>
        {
            #("\n")
            #{
                let mut emitted_names: FxHashSet<String> = FxHashSet::default();
                for model in models {
                    emit_model_prop_members(
                        __ts,
                        summary,
                        script,
                        model,
                        generic_parameters,
                        &mut emitted_names,
                    );
                }
            }
        }
    }
}

pub(super) fn append_macro_props_type_literal(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    script: Option<&str>,
    models: &[ModelDefinition],
    generic_parameters: Option<&GenericParameters>,
    prefer_identifier_props: bool,
) {
    ts! { &mut *ts =>
        {
            #("\n")
            #{
                let mut emitted_names: FxHashSet<String> = FxHashSet::default();
                let docs = summary
                    .macros
                    .define_props()
                    .filter(|call| call.type_args.is_none())
                    .and_then(|call| call.runtime_args.as_deref())
                    .map(super::runtime_docs::runtime_prop_docs)
                    .unwrap_or_default();
                for prop in summary.macros.props() {
                    let prop_type = prop.prop_type.as_deref().unwrap_or("unknown");
                    let optional = if parent_requires_prop(prop) { "" } else { "?" };
                    if let Some((_, doc)) = docs
                        .iter()
                        .find(|(key, _)| super::runtime_docs::names_prop(key, &prop.name))
                    {
                        __ts.push_str("  ");
                        __ts.push_str(doc);
                        __ts.push('\n');
                    }
                    emit_type_member(
                        __ts,
                        prop.name.as_str(),
                        optional,
                        prop_type,
                        prefer_identifier_props,
                    );
                    emitted_names.insert(prop.name.as_str().into());
                }
                for model in models {
                    emit_model_prop_members(
                        __ts,
                        summary,
                        script,
                        model,
                        generic_parameters,
                        &mut emitted_names,
                    );
                }
            }
        }
    }
}
