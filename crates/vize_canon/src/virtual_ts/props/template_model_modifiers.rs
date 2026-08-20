use vize_carton::{String, cstr};
use vize_croquis::{Croquis, macros::ModelDefinition};

pub(super) fn model_modifier_type<'a>(summary: &'a Croquis, model: &ModelDefinition) -> &'a str {
    summary
        .macros
        .model_modifier_type(model.name.as_str())
        .unwrap_or("string")
}

pub(super) fn model_modifier_prop_name(name: &str) -> String {
    if name == "modelValue" {
        String::from("modelModifiers")
    } else {
        cstr!("{name}Modifiers")
    }
}
