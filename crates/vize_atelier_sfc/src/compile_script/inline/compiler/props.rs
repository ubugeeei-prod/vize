use vize_carton::String;

use crate::script::ScriptCompileContext;

use super::super::super::props::{
    add_null_to_runtime_type, extract_prop_types_from_type, extract_with_defaults_defaults,
    resolve_prop_js_type,
};
use super::{super::type_handling::resolve_type_args, model::collect_model_infos};

/// Build props and emits definition buffer from context macros.
pub(super) fn build_props_emits(
    ctx: &ScriptCompileContext,
    _is_ts: bool,
    needs_prop_type: bool,
    needs_merge_defaults: bool,
) -> Vec<u8> {
    let mut props_emits_buf: Vec<u8> = Vec::new();

    // Extract defaults from withDefaults if present
    let with_defaults_args = ctx
        .macros
        .with_defaults
        .as_ref()
        .map(|wd| extract_with_defaults_defaults(&wd.args));

    // Collect model names from defineModel calls (needed before props)
    let model_infos: Vec<(String, String, Option<String>)> = collect_model_infos(ctx);

    if let Some(ref props_macro) = ctx.macros.define_props {
        if let Some(ref type_args) = props_macro.type_args {
            // Resolve type references (interface/type alias names) to their definitions
            let resolved_type_args =
                resolve_type_args(type_args, &ctx.interfaces, &ctx.type_aliases);
            let prop_types = extract_prop_types_from_type(&resolved_type_args);
            if !prop_types.is_empty() || !model_infos.is_empty() {
                props_emits_buf.extend_from_slice(b"  props: {\n");
                let total_items = prop_types.len() + model_infos.len();
                let mut item_idx = 0;
                for (name, prop_type) in &prop_types {
                    item_idx += 1;
                    // Try to resolve type references for props that resolved to `null`
                    let resolved_js_type = if prop_type.js_type == "null" {
                        if let Some(ref ts_type) = prop_type.ts_type {
                            resolve_prop_js_type(ts_type, &ctx.interfaces, &ctx.type_aliases)
                                .unwrap_or_else(|| prop_type.js_type.clone())
                        } else {
                            prop_type.js_type.clone()
                        }
                    } else {
                        prop_type.js_type.clone()
                    };
                    let runtime_js_type =
                        add_null_to_runtime_type(&resolved_js_type, prop_type.nullable);
                    props_emits_buf.extend_from_slice(b"    ");
                    props_emits_buf.extend_from_slice(name.as_bytes());
                    props_emits_buf.extend_from_slice(b": { type: ");
                    props_emits_buf.extend_from_slice(runtime_js_type.as_bytes());
                    if needs_prop_type {
                        if let Some(ref ts_type) = prop_type.ts_type {
                            if resolved_js_type == "null" {
                                props_emits_buf.extend_from_slice(b" as unknown as PropType<");
                            } else {
                                props_emits_buf.extend_from_slice(b" as PropType<");
                            }
                            // Normalize multi-line types to single line
                            let normalized =
                                ts_type.split_whitespace().collect::<Vec<_>>().join(" ");
                            props_emits_buf.extend_from_slice(normalized.as_bytes());
                            props_emits_buf.push(b'>');
                        }
                    }
                    props_emits_buf.extend_from_slice(b", required: ");
                    props_emits_buf.extend_from_slice(if prop_type.optional {
                        b"false"
                    } else {
                        b"true"
                    });
                    let mut has_default = false;
                    if let Some(ref defaults) = with_defaults_args {
                        if let Some(default_val) = defaults.get(name.as_str()) {
                            props_emits_buf.extend_from_slice(b", default: ");
                            props_emits_buf.extend_from_slice(default_val.as_bytes());
                            has_default = true;
                        }
                    }
                    if !has_default {
                        if let Some(ref destructure) = ctx.macros.props_destructure {
                            if let Some(binding) = destructure.bindings.get(name.as_str()) {
                                if let Some(ref default_val) = binding.default {
                                    props_emits_buf.extend_from_slice(b", default: ");
                                    props_emits_buf.extend_from_slice(default_val.as_bytes());
                                }
                            }
                        }
                    }
                    props_emits_buf.extend_from_slice(b" }");
                    if item_idx < total_items {
                        props_emits_buf.push(b',');
                    }
                    props_emits_buf.push(b'\n');
                }
                for (model_name, _, options) in &model_infos {
                    props_emits_buf.extend_from_slice(b"    \"");
                    props_emits_buf.extend_from_slice(model_name.as_bytes());
                    props_emits_buf.extend_from_slice(b"\": ");
                    if let Some(opts) = options {
                        props_emits_buf.extend_from_slice(opts.as_bytes());
                    } else {
                        props_emits_buf.extend_from_slice(b"{}");
                    }
                    props_emits_buf.extend_from_slice(b",\n");
                }
                // Remove trailing comma from last prop
                if props_emits_buf.ends_with(b",\n") {
                    let len = props_emits_buf.len();
                    props_emits_buf[len - 2] = b'\n';
                    props_emits_buf.truncate(len - 1);
                }
                props_emits_buf.extend_from_slice(b"  },\n");
            }
        } else if !props_macro.args.is_empty() {
            if needs_merge_defaults {
                let destructure = ctx.macros.props_destructure.as_ref().unwrap();
                props_emits_buf.extend_from_slice(b"  props: /*@__PURE__*/_mergeDefaults(");
                props_emits_buf.extend_from_slice(props_macro.args.as_bytes());
                props_emits_buf.extend_from_slice(b", {\n");
                let defaults: Vec<_> = destructure
                    .bindings
                    .iter()
                    .filter_map(|(k, b)| b.default.as_ref().map(|d| (k.as_str(), d.as_str())))
                    .collect();
                for (i, (key, default_val)) in defaults.iter().enumerate() {
                    props_emits_buf.extend_from_slice(b"  ");
                    props_emits_buf.extend_from_slice(key.as_bytes());
                    props_emits_buf.extend_from_slice(b": ");
                    props_emits_buf.extend_from_slice(default_val.as_bytes());
                    if i < defaults.len() - 1 {
                        props_emits_buf.push(b',');
                    }
                    props_emits_buf.push(b'\n');
                }
                props_emits_buf.extend_from_slice(b"}),\n");
            } else {
                props_emits_buf.extend_from_slice(b"  props: ");
                props_emits_buf.extend_from_slice(props_macro.args.as_bytes());
                props_emits_buf.extend_from_slice(b",\n");
            }
        }
    }

    props_emits_buf
}
