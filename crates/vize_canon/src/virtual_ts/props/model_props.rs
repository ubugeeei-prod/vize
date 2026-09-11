use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, macros::ModelDefinition};

use crate::virtual_ts::helpers::push_ts_string_literal;

use super::template_model_modifiers::{model_modifier_prop_name, model_modifier_type};

fn model_prop_type(model: &ModelDefinition) -> &str {
    model.model_type.as_deref().unwrap_or("unknown")
}

fn emit_type_member(ts: &mut String, name: &str, optional: &str, ty: &str) {
    ts.push_str("  ");
    push_ts_string_literal(ts, name);
    append!(*ts, "{optional}: {ty};\n");
}

fn emit_model_prop_members(
    ts: &mut String,
    summary: &Croquis,
    model: &ModelDefinition,
    emitted_names: &mut FxHashSet<String>,
) {
    let optional = if model.required { "" } else { "?" };
    let name = model.name.as_str();
    let prop_type = model_prop_type(model);
    if emitted_names.insert(name.into()) {
        emit_type_member(ts, name, optional, prop_type);
    }

    let modifiers_name = model_modifier_prop_name(name);
    let modifier_type = model_modifier_type(summary, model);
    if emitted_names.insert(modifiers_name.clone()) {
        let modifier_type = cstr!("Partial<Record<{modifier_type}, true>>");
        emit_type_member(ts, modifiers_name.as_str(), "?", modifier_type.as_str());
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
    ts: &mut String,
    summary: &Croquis,
    models: &[ModelDefinition],
) {
    ts.push_str("{\n");
    let mut emitted_names: FxHashSet<String> = FxHashSet::default();
    for model in models {
        emit_model_prop_members(ts, summary, model, &mut emitted_names);
    }
    ts.push('}');
}

pub(super) fn append_macro_props_type_literal(
    ts: &mut String,
    summary: &Croquis,
    models: &[ModelDefinition],
) {
    ts.push_str("{\n");
    let mut emitted_names: FxHashSet<String> = FxHashSet::default();
    for prop in summary.macros.props() {
        let prop_type = prop.prop_type.as_deref().unwrap_or("unknown");
        let optional = if prop.required { "" } else { "?" };
        emit_type_member(ts, prop.name.as_str(), optional, prop_type);
        emitted_names.insert(prop.name.as_str().into());
    }
    for model in models {
        emit_model_prop_members(ts, summary, model, &mut emitted_names);
    }
    ts.push('}');
}
