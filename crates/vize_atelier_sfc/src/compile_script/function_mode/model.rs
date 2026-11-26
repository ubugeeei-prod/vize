//! `defineModel` code generation for function mode.

use vize_carton::String;
use vize_carton::string_builder::TextBuf;

use crate::compile_script::erasure::Erasure;
use crate::script::{ScriptCompileContext, define_model_metadata, model_modifiers_binding_name};

/// Emit `defineModel` bindings and return their binding names.
pub(super) fn emit_model_bindings(
    output: &mut TextBuf,
    ctx: &ScriptCompileContext,
    mut erasure: Option<&mut Erasure>,
) -> Vec<String> {
    let mut binding_names = Vec::new();
    for model_call in &ctx.macros.define_models {
        if let Some(ref binding_name) = model_call.binding_name {
            let mut metadata = define_model_metadata(ctx.source.as_str(), model_call);
            if let Some(erasure) = erasure.as_deref_mut() {
                erasure.optional_expression(&mut metadata.runtime_options);
            }
            output.push_str("  const ");
            if let Some(ref modifiers) =
                model_modifiers_binding_name(ctx.source.as_str(), model_call)
            {
                output.push('[');
                output.push_str(binding_name);
                output.push_str(", ");
                output.push_str(modifiers);
                output.push(']');
            } else {
                output.push_str(binding_name);
            }
            output.push_str(" = _useModel(__props, \"");
            output.push_str(&metadata.name);
            output.push('"');
            if let Some(options) = metadata.runtime_options {
                output.push_str(", ");
                output.push_str(&options);
            }
            output.push_str(")\n");
            binding_names.push(String::from(binding_name.as_str()));
        }
    }
    binding_names
}
