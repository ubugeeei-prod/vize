use vize_carton::String;

use crate::script::{PropsDestructuredBindings, ScriptCompileContext};

use super::super::super::props::{
    add_null_to_runtime_type, extract_prop_types_from_type_with_context,
    extract_with_defaults_defaults, normalize_destructure_default_value, resolve_prop_js_type,
    runtime_prop_key,
};
use super::super::type_handling::resolve_type_args;

/// Build props definition buffer from context macros.
///
/// This emits the full `  props: <decl>,\n` line for the case where there is no
/// `defineModel` call. When a model is present the runtime props declaration is
/// produced by [`build_user_props_decl`] and merged with the model props via
/// `mergeModels` in `model::build_model_props_emits`.
pub(super) fn build_props_emits(
    ctx: &ScriptCompileContext,
    is_ts: bool,
    needs_prop_type: bool,
    needs_merge_defaults: bool,
    is_prod: bool,
) -> Vec<u8> {
    let mut props_emits_buf: Vec<u8> = Vec::new();

    // When defineModel is present, model::build_model_props_emits owns the
    // `props:` emission (so it can wrap with mergeModels).
    if !ctx.macros.define_models.is_empty() {
        return props_emits_buf;
    }

    if let Some(decl) =
        build_user_props_decl(ctx, is_ts, needs_prop_type, needs_merge_defaults, is_prod)
    {
        props_emits_buf.extend_from_slice(b"  props: ");
        props_emits_buf.extend_from_slice(decl.as_bytes());
        props_emits_buf.extend_from_slice(b",\n");
    }

    props_emits_buf
}

/// Build the runtime props declaration string from `defineProps` (the value that
/// goes after `props:`), without the surrounding `props: ` / `,\n`.
///
/// Returns `None` when there is no `defineProps` call (or it produced nothing).
pub(super) fn build_user_props_decl(
    ctx: &ScriptCompileContext,
    _is_ts: bool,
    needs_prop_type: bool,
    needs_merge_defaults: bool,
    is_prod: bool,
) -> Option<String> {
    let props_macro = ctx.macros.define_props.as_ref()?;

    // Extract defaults from withDefaults if present
    let with_defaults_args = ctx
        .macros
        .with_defaults
        .as_ref()
        .map(|wd| extract_with_defaults_defaults(&wd.args));

    let mut decl: Vec<u8> = Vec::new();

    if let Some(ref type_args) = props_macro.type_args {
        // Resolve type references (interface/type alias names) to their definitions
        let resolved_type_args = resolve_type_args(type_args, &ctx.interfaces, &ctx.type_aliases);
        let prop_types = extract_prop_types_from_type_with_context(
            &resolved_type_args,
            Some(&ctx.interfaces),
            Some(&ctx.type_aliases),
        );
        if prop_types.is_empty() {
            if let Some(ref destructure) = ctx.macros.props_destructure {
                build_unknown_type_destructured_props_decl(&mut decl, destructure);
            } else {
                return None;
            }
        } else {
            decl.extend_from_slice(b"{\n");
            let total_items = prop_types.len();
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
                decl.extend_from_slice(b"    ");
                let key = runtime_prop_key(name);
                decl.extend_from_slice(key.as_bytes());
                let mut has_option = false;
                if is_prod && runtime_js_type != "Boolean" {
                    decl.extend_from_slice(b": {");
                } else {
                    decl.extend_from_slice(b": { type: ");
                    decl.extend_from_slice(runtime_js_type.as_bytes());
                    has_option = true;
                    if needs_prop_type && let Some(ref ts_type) = prop_type.ts_type {
                        if resolved_js_type == "null" {
                            decl.extend_from_slice(b" as unknown as PropType<");
                        } else {
                            decl.extend_from_slice(b" as PropType<");
                        }
                        // Normalize multi-line types to single line
                        let normalized = ts_type.split_whitespace().collect::<Vec<_>>().join(" ");
                        decl.extend_from_slice(normalized.as_bytes());
                        decl.push(b'>');
                    }
                }
                if !is_prod {
                    // Vue's type-only inference emits `required` explicitly
                    // for both branches in dev mode so prop-validation
                    // warnings fire for required props that are omitted. The
                    // previous output skipped `required: true` entirely. (#967)
                    if has_option {
                        decl.extend_from_slice(b", ");
                    } else {
                        decl.push(b' ');
                    }
                    if prop_type.optional {
                        decl.extend_from_slice(b"required: false");
                    } else {
                        decl.extend_from_slice(b"required: true");
                    }
                    has_option = true;
                }
                let mut has_default = false;
                if let Some(ref defaults) = with_defaults_args
                    && let Some(default_val) = defaults.get(name.as_str())
                {
                    if has_option {
                        decl.extend_from_slice(b", ");
                    } else {
                        decl.push(b' ');
                    }
                    decl.extend_from_slice(b"default: ");
                    decl.extend_from_slice(default_val.as_bytes());
                    has_option = true;
                    has_default = true;
                }
                if !has_default
                    && let Some(ref destructure) = ctx.macros.props_destructure
                    && let Some(binding) = destructure.bindings.get(name.as_str())
                    && let Some(ref default_val) = binding.default
                {
                    if has_option {
                        decl.extend_from_slice(b", ");
                    } else {
                        decl.push(b' ');
                    }
                    decl.extend_from_slice(b"default: ");
                    let default_val = normalize_destructure_default_value(default_val);
                    decl.extend_from_slice(default_val.as_bytes());
                }
                if has_option {
                    decl.extend_from_slice(b" }");
                } else {
                    decl.push(b'}');
                }
                if item_idx < total_items {
                    decl.push(b',');
                }
                decl.push(b'\n');
            }
            decl.extend_from_slice(b"  }");
        }
    } else if !props_macro.args.is_empty() {
        if let (true, Some(destructure)) =
            (needs_merge_defaults, ctx.macros.props_destructure.as_ref())
        {
            decl.extend_from_slice(b"/*@__PURE__*/_mergeDefaults(");
            decl.extend_from_slice(props_macro.args.as_bytes());
            decl.extend_from_slice(b", {\n");
            // Iterate in source declaration order (matches Vue's iteration order
            // over the destructured bindings).
            let defaults: Vec<(
                &str,
                &super::super::super::super::script::PropsDestructureBinding,
            )> = destructure
                .keys
                .iter()
                .filter_map(|k| {
                    destructure
                        .bindings
                        .get(k.as_str())
                        .and_then(|b| b.default.as_ref().map(|_| (k.as_str(), b)))
                })
                .collect();
            for (i, (key, binding)) in defaults.iter().enumerate() {
                let default_val = binding.default.as_deref().unwrap_or_default();
                decl.extend_from_slice(b"  ");
                decl.extend_from_slice(key.as_bytes());
                decl.extend_from_slice(b": ");
                if binding.default_needs_factory {
                    // Wrap non-literal expressions in a factory: `() => (expr)`.
                    decl.extend_from_slice(b"() => (");
                    decl.extend_from_slice(default_val.trim().as_bytes());
                    decl.push(b')');
                } else {
                    // Literals, bare identifiers and functions are emitted as-is.
                    decl.extend_from_slice(default_val.trim().as_bytes());
                }
                if binding.default_skip_factory {
                    decl.extend_from_slice(b", __skip_");
                    decl.extend_from_slice(key.as_bytes());
                    decl.extend_from_slice(b": true");
                }
                if i < defaults.len() - 1 {
                    decl.push(b',');
                }
                decl.push(b'\n');
            }
            decl.extend_from_slice(b"})");
        } else {
            decl.extend_from_slice(props_macro.args.as_bytes());
        }
    } else {
        return None;
    }

    // SAFETY: assembled from UTF-8 source slices and ASCII glue only.
    #[allow(clippy::disallowed_types)]
    let s = unsafe { std::string::String::from_utf8_unchecked(decl) };
    Some(s.into())
}

fn build_unknown_type_destructured_props_decl(
    decl: &mut Vec<u8>,
    destructure: &PropsDestructuredBindings,
) {
    decl.extend_from_slice(b"{\n");
    for (i, key) in destructure.keys.iter().enumerate() {
        decl.extend_from_slice(b"    ");
        decl.extend_from_slice(key.as_bytes());
        if let Some(binding) = destructure.bindings.get(key.as_str())
            && let Some(default_val) = binding.default.as_ref()
        {
            decl.extend_from_slice(b": { default: ");
            let default_val = normalize_destructure_default_value(default_val);
            decl.extend_from_slice(default_val.as_bytes());
            decl.extend_from_slice(b" }");
        } else {
            decl.extend_from_slice(b": {}");
        }
        if i < destructure.keys.len() - 1 {
            decl.push(b',');
        }
        decl.push(b'\n');
    }
    decl.extend_from_slice(b"  }");
}
