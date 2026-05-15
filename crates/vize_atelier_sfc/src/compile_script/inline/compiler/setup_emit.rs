use std::borrow::Cow;

use vize_carton::{profile, String};

use crate::script::ScriptCompileContext;

use super::await_transform::transform_await_expressions;

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_setup_body(
    output: &mut vize_carton::Vec<u8>,
    ctx: &ScriptCompileContext,
    model_infos: &[(String, String, Option<String>)],
    setup_body_lines: &[String],
    source_is_ts: bool,
    _is_ts: bool,
    is_async: bool,
    css_vars: &[Cow<'_, str>],
    scope_id: &str,
    has_css_vars: bool,
) {
    // Emit binding: const emit = __emit
    if let Some(ref emits_macro) = ctx.macros.define_emits {
        if let Some(ref binding_name) = emits_macro.binding_name {
            output.extend_from_slice(b"const ");
            output.extend_from_slice(binding_name.as_bytes());
            output.extend_from_slice(b" = __emit\n");
        }
    }

    // Props binding: const props = __props
    if let Some(ref props_macro) = ctx.macros.define_props {
        if let Some(ref binding_name) = props_macro.binding_name {
            output.extend_from_slice(b"const ");
            output.extend_from_slice(binding_name.as_bytes());
            output.extend_from_slice(b" = __props\n");
        }
    }

    // Model bindings: const model = _useModel(__props, 'modelValue')
    if !model_infos.is_empty() {
        for (model_name, binding_name, _) in model_infos {
            output.extend_from_slice(b"const ");
            output.extend_from_slice(binding_name.as_bytes());
            output.extend_from_slice(b" = _useModel(__props, \"");
            output.extend_from_slice(model_name.as_bytes());
            output.extend_from_slice(b"\")\n");
        }
    }

    // Slots binding: const slots = _useSlots()
    if let Some(ref slots_macro) = ctx.macros.define_slots {
        if let Some(ref binding_name) = slots_macro.binding_name {
            output.extend_from_slice(b"const ");
            output.extend_from_slice(binding_name.as_bytes());
            output.extend_from_slice(b" = _useSlots()\n");
        }
    }

    // Output setup code lines (non-hoisted), transforming await expressions for async setup
    if is_async {
        let transformed_async = profile!(
            "atelier.script_inline.transform_await",
            transform_await_expressions(setup_body_lines, source_is_ts)
        );
        for line in &transformed_async {
            output.extend_from_slice(line.as_bytes());
            output.push(b'\n');
        }
    } else {
        for line in setup_body_lines {
            output.extend_from_slice(line.as_bytes());
            output.push(b'\n');
        }
    }

    // defineExpose: transform to __expose(...)
    if let Some(ref expose_macro) = ctx.macros.define_expose {
        let args = expose_macro.args.trim();
        output.extend_from_slice(b"__expose(");
        output.extend_from_slice(args.as_bytes());
        output.extend_from_slice(b")\n");
    }

    // useCssVars injection for v-bind() in <style>
    if has_css_vars {
        output.extend_from_slice(b"_useCssVars((_ctx) => ({\n");
        for (i, var_expr) in css_vars.iter().enumerate() {
            output.extend_from_slice(b"  \"");
            output.extend_from_slice(scope_id.as_bytes());
            output.extend_from_slice(b"-");
            output.extend_from_slice(var_expr.as_bytes());
            output.extend_from_slice(b"\": (_unref(");
            output.extend_from_slice(var_expr.as_bytes());
            output.extend_from_slice(b"))");
            if i < css_vars.len() - 1 {
                output.extend_from_slice(b",");
            }
            output.extend_from_slice(b"\n");
        }
        output.extend_from_slice(b"}))\n");
    }
}
