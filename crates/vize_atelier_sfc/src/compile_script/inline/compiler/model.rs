use vize_carton::{String, ToCompactString, cstr};

use crate::script::ScriptCompileContext;

use super::super::super::props::{
    extract_emit_names_from_type, resolve_prop_js_type, ts_type_to_js_type,
};

/// Resolved info for a single `defineModel` call.
#[derive(Debug, Clone)]
pub(super) struct ModelInfo {
    /// Model (prop) name, e.g. `modelValue` or `title`.
    pub name: String,
    /// Local binding name the model is assigned to.
    pub binding_name: String,
    /// Runtime options object source (`{ ... }`), if provided.
    pub options: Option<String>,
    /// Explicit `<T>` type-argument source, if provided.
    pub type_arg: Option<String>,
}

/// Resolve a defineModel `<T>` type argument to its runtime constructor,
/// mirroring `@vue/compiler-sfc`'s `inferRuntimeType`: primitives map directly
/// (`string` → `String`), and local interface / type-alias references that do
/// not map directly resolve to `Object` via the script context.
fn model_runtime_type(ctx: &ScriptCompileContext, type_arg: &str) -> String {
    let js_type = ts_type_to_js_type(type_arg);
    if js_type == "null" {
        resolve_prop_js_type(type_arg, &ctx.interfaces, &ctx.type_aliases).unwrap_or(js_type)
    } else {
        js_type
    }
}

/// Render the runtime prop-options object for a single defineModel, mirroring
/// `@vue/compiler-sfc`'s `genModelProps`: an explicit `<T>` type argument is
/// resolved to its runtime constructor as `{ type: <RuntimeType> }`, merged
/// with any runtime options as `, ...{ opts }`. Without a type argument the
/// options object (or `{}`) is emitted verbatim.
pub(super) fn model_value_prop(ctx: &ScriptCompileContext, info: &ModelInfo) -> String {
    let type_arg = info
        .type_arg
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty());
    match (type_arg, info.options.as_deref()) {
        (Some(type_arg), Some(options)) => {
            let runtime_type = model_runtime_type(ctx, type_arg);
            cstr!("{{ type: {}, ...{} }}", runtime_type, options.trim())
        }
        (Some(type_arg), None) => {
            let runtime_type = model_runtime_type(ctx, type_arg);
            cstr!("{{ type: {} }}", runtime_type)
        }
        (None, Some(options)) => options.trim().to_compact_string(),
        (None, None) => "{}".to_compact_string(),
    }
}

/// Build model-specific props and emits when defineModel is used without defineProps,
/// plus the emits array combining defineEmits and defineModel emits.
pub(super) fn build_model_props_emits(
    ctx: &ScriptCompileContext,
    model_infos: &[ModelInfo],
    _is_ts: bool,
    _needs_prop_type: bool,
    _needs_merge_defaults: bool,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    if !model_infos.is_empty() && ctx.macros.define_props.is_none() {
        buf.extend_from_slice(b"  props: {\n");
        for info in model_infos {
            // Model value prop
            buf.extend_from_slice(b"    \"");
            buf.extend_from_slice(info.name.as_bytes());
            buf.extend_from_slice(b"\": ");
            buf.extend_from_slice(model_value_prop(ctx, info).as_bytes());
            buf.extend_from_slice(b",\n");
            // Model modifiers prop: "modelModifiers" for default, "<name>Modifiers" for named
            buf.extend_from_slice(b"    \"");
            if info.name == "modelValue" {
                buf.extend_from_slice(b"modelModifiers");
            } else {
                buf.extend_from_slice(info.name.as_bytes());
                buf.extend_from_slice(b"Modifiers");
            }
            buf.extend_from_slice(b"\": {},\n");
        }
        buf.extend_from_slice(b"  },\n");
    }

    // Emits definition - combine defineEmits and defineModel emits
    let mut all_emits: Vec<String> = Vec::new();
    if let Some(ref emits_macro) = ctx.macros.define_emits {
        if !emits_macro.args.is_empty() {
            let args = emits_macro.args.trim();
            if args.starts_with('[') && args.ends_with(']') {
                let inner = &args[1..args.len() - 1];
                for part in inner.split(',') {
                    let name = part.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !name.is_empty() {
                        all_emits.push(name.to_compact_string());
                    }
                }
            }
        } else if let Some(ref type_args) = emits_macro.type_args {
            let emit_names = extract_emit_names_from_type(type_args);
            all_emits.extend(emit_names);
        }
    }
    for info in model_infos {
        let mut name = String::with_capacity(7 + info.name.len());
        name.push_str("update:");
        name.push_str(&info.name);
        all_emits.push(name);
    }
    if !all_emits.is_empty() {
        buf.extend_from_slice(b"  emits: [");
        for (i, name) in all_emits.iter().enumerate() {
            if i > 0 {
                buf.extend_from_slice(b", ");
            }
            buf.push(b'"');
            buf.extend_from_slice(name.as_bytes());
            buf.push(b'"');
        }
        buf.extend_from_slice(b"],\n");
    }

    buf
}

/// Collect model info from defineModel calls.
pub(super) fn collect_model_infos(ctx: &ScriptCompileContext) -> Vec<ModelInfo> {
    ctx.macros
        .define_models
        .iter()
        .map(|m| {
            let name = if m.args.trim().is_empty() {
                "modelValue".to_compact_string()
            } else {
                let args = m.args.trim();
                if args.starts_with('\'') || args.starts_with('"') {
                    args.trim_matches(|c| c == '\'' || c == '"')
                        .split(',')
                        .next()
                        .unwrap_or("modelValue")
                        .trim_matches(|c| c == '\'' || c == '"')
                        .to_compact_string()
                } else {
                    "modelValue".to_compact_string()
                }
            };
            let binding_name = m
                .binding_name
                .as_deref()
                .map(String::from)
                .unwrap_or_else(|| name.clone());
            let options = if m.args.trim().is_empty() {
                None
            } else {
                let args = m.args.trim();
                if args.starts_with('{') {
                    Some(args.to_compact_string())
                } else if args.contains(',') {
                    args.split_once(',')
                        .map(|(_, opts)| opts.trim().to_compact_string())
                } else {
                    None
                }
            };
            ModelInfo {
                name,
                binding_name,
                options,
                type_arg: m.type_args.clone(),
            }
        })
        .collect()
}
