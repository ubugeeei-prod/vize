//! `defineModel` code generation for function mode.

use vize_carton::String;

use crate::script::{ScriptCompileContext, define_model_metadata, model_modifiers_binding_name};

/// Emit `defineModel` bindings and return their binding names.
pub(super) fn emit_model_bindings(
    output: &mut vize_carton::Vec<u8>,
    ctx: &ScriptCompileContext,
) -> Vec<String> {
    let mut binding_names = Vec::new();
    for model_call in &ctx.macros.define_models {
        if let Some(ref binding_name) = model_call.binding_name {
            let metadata = define_model_metadata(ctx.source.as_str(), model_call);
            output.extend_from_slice(b"  const ");
            if let Some(ref modifiers) =
                model_modifiers_binding_name(ctx.source.as_str(), model_call)
            {
                output.push(b'[');
                output.extend_from_slice(binding_name.as_bytes());
                output.extend_from_slice(b", ");
                output.extend_from_slice(modifiers.as_bytes());
                output.push(b']');
            } else {
                output.extend_from_slice(binding_name.as_bytes());
            }
            output.extend_from_slice(b" = _useModel(__props, \"");
            output.extend_from_slice(metadata.name.as_bytes());
            output.push(b'"');
            if let Some(options) = metadata.runtime_options {
                output.extend_from_slice(b", ");
                output.extend_from_slice(options.as_bytes());
            }
            output.extend_from_slice(b")\n");
            binding_names.push(String::from(binding_name.as_str()));
        }
    }
    binding_names
}
