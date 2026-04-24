use vize_carton::{String, ToCompactString};

use crate::script::ScriptCompileContext;

use super::super::super::props::extract_emit_names_from_type;

/// Build model-specific props and emits when defineModel is used without defineProps,
/// plus the emits array combining defineEmits and defineModel emits.
pub(super) fn build_model_props_emits(
    ctx: &ScriptCompileContext,
    model_infos: &[(String, String, Option<String>)],
    _is_ts: bool,
    _needs_prop_type: bool,
    _needs_merge_defaults: bool,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    if !model_infos.is_empty() && ctx.macros.define_props.is_none() {
        buf.extend_from_slice(b"  props: {\n");
        for (model_name, _binding_name, options) in model_infos {
            // Model value prop
            buf.extend_from_slice(b"    \"");
            buf.extend_from_slice(model_name.as_bytes());
            buf.extend_from_slice(b"\": ");
            if let Some(opts) = options {
                buf.extend_from_slice(opts.as_bytes());
            } else {
                buf.extend_from_slice(b"{}");
            }
            buf.extend_from_slice(b",\n");
            // Model modifiers prop: "modelModifiers" for default, "<name>Modifiers" for named
            buf.extend_from_slice(b"    \"");
            if model_name == "modelValue" {
                buf.extend_from_slice(b"modelModifiers");
            } else {
                buf.extend_from_slice(model_name.as_bytes());
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
    for (model_name, _, _) in model_infos {
        let mut name = String::with_capacity(7 + model_name.len());
        name.push_str("update:");
        name.push_str(model_name);
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
///
/// Returns Vec of (model_name, binding_name, options).
pub(super) fn collect_model_infos(
    ctx: &ScriptCompileContext,
) -> Vec<(String, String, Option<String>)> {
    ctx.macros
        .define_models
        .iter()
        .map(|m| {
            let model_name = if m.args.trim().is_empty() {
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
                .unwrap_or_else(|| model_name.clone());
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
            (model_name, binding_name, options)
        })
        .collect()
}
