//! Component option and setup-helper emission.

use vize_carton::{String, ToCompactString};

use crate::script::{
    PropsDestructuredBindings, ScriptCompileContext, define_model_name,
    model_modifiers_binding_name, resolve_type_args,
};

use super::super::super::props::{
    PropTypeInfo, add_null_to_runtime_type, extract_emit_names_from_type,
    extract_prop_types_from_type_with_context, normalize_destructure_default_value,
    resolve_prop_js_type, runtime_prop_key,
};

/// Emit props definition to the output buffer.
///
/// Handles regular defineProps, destructured props with defaults, and type-based props.
pub(super) fn emit_props_definition(
    output: &mut vize_carton::Vec<u8>,
    ctx: &ScriptCompileContext,
    has_props_destructure: bool,
    needs_prop_type: bool,
    _is_ts: bool,
) {
    let with_defaults = super::super::prop_defaults::RuntimePropDefaults::new(ctx);
    if let (true, Some(destructure)) =
        (has_props_destructure, ctx.macros.props_destructure.as_ref())
    {
        // Check if there are any defaults
        let has_defaults = destructure.bindings.values().any(|b| b.default.is_some());

        if has_defaults {
            // Use mergeDefaults format: _mergeDefaults(runtimeProps, { prop2: default })
            // Get the original props argument from defineProps (or generate from type args)
            let original_props: String = if let Some(ref props_macro) = ctx.macros.define_props {
                if let Some(ref type_args) = props_macro.type_args {
                    let prop_types = prop_types_from_context(ctx, type_args);
                    if prop_types.is_empty() {
                        if let Some(ref destructure) = ctx.macros.props_destructure {
                            destructured_props_runtime_decl(destructure)
                        } else {
                            "{}".to_compact_string()
                        }
                    } else {
                        let mut names: Vec<_> =
                            prop_types.iter().map(|(n, _)| n.as_str()).collect();
                        names.sort();
                        let mut s = String::from("{ ");
                        for (i, name) in names.iter().enumerate() {
                            if i > 0 {
                                s.push_str(", ");
                            }
                            let key = runtime_prop_key(name);
                            s.push_str(key.as_str());
                            s.push_str(": {}");
                        }
                        s.push_str(" }");
                        s
                    }
                } else if !props_macro.args.is_empty() {
                    String::from(props_macro.args.as_str())
                } else {
                    "[]".to_compact_string()
                }
            } else {
                "[]".to_compact_string()
            };

            output.extend_from_slice(b"  props: /*@__PURE__*/_mergeDefaults(");
            output.extend_from_slice(original_props.as_bytes());
            output.extend_from_slice(b", {\n");

            // Add defaults
            for (key, binding) in &destructure.bindings {
                if let Some(ref default_val) = binding.default {
                    output.extend_from_slice(b"  ");
                    output.extend_from_slice(key.as_bytes());
                    output.extend_from_slice(b": ");
                    let default_val = normalize_destructure_default_value(default_val);
                    output.extend_from_slice(default_val.as_bytes());
                    output.push(b'\n');
                }
            }
            output.extend_from_slice(b"}),\n");
        } else {
            // No defaults - just use the original props array
            if let Some(ref props_macro) = ctx.macros.define_props
                && !props_macro.args.is_empty()
            {
                output.extend_from_slice(b"  props: ");
                output.extend_from_slice(props_macro.args.as_bytes());
                output.extend_from_slice(b",\n");
            } else if let Some(ref props_macro) = ctx.macros.define_props
                && props_macro.type_args.is_some()
            {
                let prop_types = props_macro
                    .type_args
                    .as_ref()
                    .map(|type_args| prop_types_from_context(ctx, type_args))
                    .unwrap_or_default();
                if prop_types.is_empty() {
                    output.extend_from_slice(b"  props: ");
                    output
                        .extend_from_slice(destructured_props_runtime_decl(destructure).as_bytes());
                    output.extend_from_slice(b",\n");
                } else {
                    output.extend_from_slice(b"  props: {\n");
                    let mut names: Vec<_> = prop_types.iter().map(|(n, _)| n.as_str()).collect();
                    names.sort();
                    for name in names {
                        output.extend_from_slice(b"    ");
                        let key = runtime_prop_key(name);
                        output.extend_from_slice(key.as_bytes());
                        output.extend_from_slice(b": {},\n");
                    }
                    output.extend_from_slice(b"  },\n");
                }
            }
        }
    } else if let Some(ref props_macro) = ctx.macros.define_props {
        if let Some(ref type_args) = props_macro.type_args {
            // For type-based props, extract full prop definitions
            let prop_types = prop_types_from_context(ctx, type_args);
            if !prop_types.is_empty() {
                output.extend_from_slice(b"  props: {\n");
                // Sort props for deterministic output
                let mut sorted_props: Vec<_> = prop_types.iter().collect();
                sorted_props.sort_by(|a, b| a.0.cmp(&b.0));
                for (name, prop_type) in sorted_props {
                    let resolved_js_type = resolve_prop_type(prop_type, ctx);
                    let runtime_js_type =
                        add_null_to_runtime_type(&resolved_js_type, prop_type.nullable);
                    output.extend_from_slice(b"    ");
                    let key = runtime_prop_key(name);
                    output.extend_from_slice(key.as_bytes());
                    output.extend_from_slice(b": { type: ");
                    output.extend_from_slice(runtime_js_type.as_bytes());
                    if needs_prop_type && let Some(ref ts_type) = prop_type.ts_type {
                        if resolved_js_type == "null" {
                            output.extend_from_slice(b" as unknown as PropType<");
                        } else {
                            output.extend_from_slice(b" as PropType<");
                        }
                        // Normalize multi-line types to single line
                        let normalized: String =
                            String::from(ts_type.split_whitespace().collect::<Vec<_>>().join(" "));
                        output.extend_from_slice(normalized.as_bytes());
                        output.push(b'>');
                    }
                    with_defaults.emit_contract(output, name, prop_type.optional);
                    output.extend_from_slice(b" },\n");
                }
                output.extend_from_slice(b"  },\n");
            }
        } else if !props_macro.args.is_empty() {
            output.extend_from_slice(b"  props: ");
            output.extend_from_slice(props_macro.args.as_bytes());
            output.extend_from_slice(b",\n");
        }
    }
}

pub(super) fn prop_types_from_context(
    ctx: &ScriptCompileContext,
    type_args: &str,
) -> Vec<(String, PropTypeInfo)> {
    let resolved_type_args = resolve_type_args(type_args, &ctx.interfaces, &ctx.type_aliases);
    extract_prop_types_from_type_with_context(
        &resolved_type_args,
        Some(&ctx.interfaces),
        Some(&ctx.type_aliases),
    )
}

fn resolve_prop_type(prop_type: &PropTypeInfo, ctx: &ScriptCompileContext) -> String {
    if prop_type.js_type == "null" {
        prop_type
            .ts_type
            .as_ref()
            .and_then(|ts_type| resolve_prop_js_type(ts_type, &ctx.interfaces, &ctx.type_aliases))
            .unwrap_or_else(|| prop_type.js_type.clone())
    } else {
        prop_type.js_type.clone()
    }
}

fn destructured_props_runtime_decl(destructure: &PropsDestructuredBindings) -> String {
    let mut decl = String::from("{ ");
    for (i, key) in destructure.keys.iter().enumerate() {
        if i > 0 {
            decl.push_str(", ");
        }
        decl.push_str(key.as_str());
        decl.push_str(": {}");
    }
    decl.push_str(" }");
    decl
}

/// Collect model names from defineModel calls.
pub(super) fn collect_model_names(ctx: &ScriptCompileContext) -> Vec<String> {
    ctx.macros
        .define_models
        .iter()
        .map(|m| define_model_name(ctx.source.as_str(), m))
        .collect()
}

/// Emit emits definition to the output buffer.
///
/// Combines defineEmits and defineModel emit events.
pub(super) fn emit_emits_definition(
    output: &mut vize_carton::Vec<u8>,
    ctx: &ScriptCompileContext,
    model_names: &[String],
) {
    let mut all_emits: Vec<String> = Vec::new();

    // Add emits from defineEmits
    if let Some(ref emits_macro) = ctx.macros.define_emits {
        if let Some(ref type_args) = emits_macro.type_args {
            let resolved_type_args =
                resolve_type_args(type_args, &ctx.interfaces, &ctx.type_aliases);
            let emit_names = extract_emit_names_from_type(&resolved_type_args);
            all_emits.extend(emit_names);
        } else if !emits_macro.args.is_empty() {
            // Runtime args - we'll output separately
        }
    }

    // Add update:modelValue emits from defineModel
    for model_name in model_names {
        let mut name = String::with_capacity(7 + model_name.len());
        name.push_str("update:");
        name.push_str(model_name);
        all_emits.push(name);
    }

    // Output emits
    if !all_emits.is_empty() {
        output.extend_from_slice(b"  emits: [");
        for (i, name) in all_emits.iter().enumerate() {
            if i > 0 {
                output.extend_from_slice(b", ");
            }
            output.push(b'"');
            output.extend_from_slice(name.as_bytes());
            output.push(b'"');
        }
        output.extend_from_slice(b"],\n");
    } else if let Some(ref emits_macro) = ctx.macros.define_emits
        && !emits_macro.args.is_empty()
    {
        output.extend_from_slice(b"  emits: ");
        output.extend_from_slice(emits_macro.args.as_bytes());
        output.extend_from_slice(b",\n");
    }
}

/// Emit the __expose() call to the output buffer.
pub(super) fn emit_expose(output: &mut vize_carton::Vec<u8>, ctx: &ScriptCompileContext) {
    if let Some(ref expose_macro) = ctx.macros.define_expose {
        // args contains the argument content (e.g., "{ foo, bar }")
        let args = expose_macro.args.trim();
        if args.is_empty() {
            output.extend_from_slice(b"  __expose();\n");
        } else {
            output.extend_from_slice(b"  __expose(");
            output.extend_from_slice(args.as_bytes());
            output.extend_from_slice(b");\n");
        }
    } else {
        // No defineExpose, but still need to call __expose() for Vue runtime
        output.extend_from_slice(b"  __expose();\n");
    }
}

/// Emit defineModel bindings and return the binding names.
pub(super) fn emit_model_bindings(
    output: &mut vize_carton::Vec<u8>,
    ctx: &ScriptCompileContext,
) -> Vec<String> {
    let mut model_binding_names: Vec<String> = Vec::new();
    for model_call in &ctx.macros.define_models {
        if let Some(ref binding_name) = model_call.binding_name {
            let model_name = define_model_name(ctx.source.as_str(), model_call);

            output.extend_from_slice(b"  const ");
            if let Some(ref modifiers_binding_name) =
                model_modifiers_binding_name(ctx.source.as_str(), model_call)
            {
                output.push(b'[');
                output.extend_from_slice(binding_name.as_bytes());
                output.extend_from_slice(b", ");
                output.extend_from_slice(modifiers_binding_name.as_bytes());
                output.extend_from_slice(b"]");
            } else {
                output.extend_from_slice(binding_name.as_bytes());
            }
            output.extend_from_slice(b" = _useModel(__props, \"");
            output.extend_from_slice(model_name.as_bytes());
            output.extend_from_slice(b"\")\n");
            model_binding_names.push(String::from(binding_name.as_str()));
        }
    }
    model_binding_names
}
