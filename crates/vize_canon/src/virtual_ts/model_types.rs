//! Public model defaults retain TypeScript's inference in their authored setup scope.

use vize_carton::{String, append, cstr};
use vize_croquis::{Croquis, macros::ModelDefinition};

use super::{generic_parameters::GenericParameters, helpers::push_ts_string_literal};

pub(crate) fn has_inferred_defaults(summary: &Croquis) -> bool {
    summary
        .macros
        .models()
        .iter()
        .any(|model| model.model_type.is_none() && model.default_value.is_some())
}

pub(crate) fn model_value_type(
    generic_parameters: Option<&GenericParameters>,
    model: &ModelDefinition,
) -> String {
    if let Some(ty) = &model.model_type {
        return ty.as_str().into();
    }
    if model.default_value.is_none() {
        return "unknown".into();
    }
    let arguments = generic_parameters
        .map(|generic| cstr!("<{}>", generic.names))
        .unwrap_or_default();
    let mut ty = cstr!("__VizeModelDefaults{arguments}[");
    push_ts_string_literal(&mut ty, model.name.as_str());
    ty.push(']');
    ty
}

pub(crate) fn emit_defaults_artifact(ts: &mut String, summary: &Croquis, fields: &mut Vec<String>) {
    if !has_inferred_defaults(summary) {
        return;
    }
    ts.push_str("\n  const __vize_model_defaults = {\n");
    for model in summary.macros.models() {
        if model.model_type.is_some() {
            continue;
        }
        let Some(default) = &model.default_value else {
            continue;
        };
        ts.push_str("    ");
        push_ts_string_literal(ts, model.name.as_str());
        append!(*ts, ": ({default}),\n");
    }
    ts.push_str("  };\n");
    fields.push("__vize_model_defaults".into());
}

pub(crate) fn emit_defaults_type(ts: &mut String, generic_param: Option<&GenericParameters>) {
    let (declaration, arguments) = super::generator::generics::generic_injection(generic_param)
        .map(|(declaration, names)| (cstr!("<{declaration}>"), cstr!("<{}>", names.join(", "))))
        .unwrap_or_default();
    append!(
        *ts,
        "type __VizeModelDefaults{declaration} = Awaited<ReturnType<typeof __setup{arguments}>>[\"__vize_model_defaults\"];\n"
    );
}
