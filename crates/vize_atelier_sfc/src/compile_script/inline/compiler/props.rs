use vize_carton::String;
use vize_carton::string_builder::TextBuf;

use crate::script::{PropsDestructuredBindings, ScriptCompileContext};

use super::super::super::props::{
    WithDefaultsValues, add_null_to_runtime_type, extract_prop_types_from_type_with_context,
    normalize_destructure_default_value_with_factory, resolve_prop_js_type, runtime_prop_key,
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
    with_defaults_values: Option<&WithDefaultsValues>,
    is_prod: bool,
) -> TextBuf {
    let mut props_emits_buf = TextBuf::new();

    // When defineModel is present, model::build_model_props_emits owns the
    // `props:` emission (so it can wrap with mergeModels).
    if !ctx.macros.define_models.is_empty() {
        return props_emits_buf;
    }

    if let Some(decl) = build_user_props_decl(
        ctx,
        is_ts,
        needs_prop_type,
        needs_merge_defaults,
        with_defaults_values,
        is_prod,
    ) {
        props_emits_buf.push_str("  props: ");
        props_emits_buf.push_str(&decl);
        props_emits_buf.push_str(",\n");
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
    with_defaults_values: Option<&WithDefaultsValues>,
    is_prod: bool,
) -> Option<String> {
    let props_macro = ctx.macros.define_props.as_ref()?;
    let static_defaults = with_defaults_values.map(|defaults| &defaults.static_values);

    let mut decl = TextBuf::new();

    if let Some(ref type_args) = props_macro.type_args {
        // Resolve type references (interface/type alias names) to their definitions
        let resolved_type_args = resolve_type_args(type_args, &ctx.interfaces, &ctx.type_aliases);
        let prop_types = extract_prop_types_from_type_with_context(
            &resolved_type_args,
            Some(&ctx.interfaces),
            Some(&ctx.type_aliases),
        );
        if prop_types.is_empty() {
            let destructure = ctx.macros.props_destructure.as_ref()?;
            build_unknown_type_destructured_props_decl(&mut decl, destructure);
        } else {
            decl.push_str("{\n");
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
                decl.push_str("    ");
                let key = runtime_prop_key(name);
                decl.push_str(&key);
                let mut has_option = false;
                if is_prod && !should_preserve_prod_prop_type(&runtime_js_type) {
                    decl.push_str(": {");
                } else {
                    decl.push_str(": { type: ");
                    decl.push_str(&runtime_js_type);
                    has_option = true;
                    if needs_prop_type && let Some(ref ts_type) = prop_type.ts_type {
                        if resolved_js_type == "null" {
                            decl.push_str(" as unknown as PropType<");
                        } else {
                            decl.push_str(" as PropType<");
                        }
                        // Normalize multi-line types to single line
                        let normalized = ts_type.split_whitespace().collect::<Vec<_>>().join(" ");
                        decl.push_str(&normalized);
                        decl.push('>');
                    }
                }
                if !is_prod {
                    // Vue's type-only inference emits `required` explicitly
                    // for both branches in dev mode so prop-validation
                    // warnings fire for required props that are omitted. The
                    // previous output skipped `required: true` entirely. (#967)
                    if has_option {
                        decl.push_str(", ");
                    } else {
                        decl.push(' ');
                    }
                    if prop_type.optional {
                        decl.push_str("required: false");
                    } else {
                        decl.push_str("required: true");
                    }
                    if prop_type.skip_check {
                        decl.push_str(", skipCheck: true");
                    }
                    has_option = true;
                }
                let mut has_default = false;
                if let Some(defaults) = static_defaults
                    && let Some(default_val) = defaults.get(name.as_str())
                {
                    if has_option {
                        decl.push_str(", ");
                    } else {
                        decl.push(' ');
                    }
                    decl.push_str("default: ");
                    decl.push_str(default_val);
                    has_option = true;
                    has_default = true;
                }
                if !has_default
                    && let Some(ref destructure) = ctx.macros.props_destructure
                    && let Some(binding) = destructure.bindings.get(name.as_str())
                    && let Some(ref default_val) = binding.default
                {
                    if has_option {
                        decl.push_str(", ");
                    } else {
                        decl.push(' ');
                    }
                    decl.push_str("default: ");
                    let default_val = normalize_destructure_default_value_with_factory(
                        default_val,
                        binding.default_needs_factory,
                    );
                    decl.push_str(&default_val);
                }
                if has_option {
                    decl.push_str(" }");
                } else {
                    decl.push('}');
                }
                if item_idx < total_items {
                    decl.push(',');
                }
                decl.push('\n');
            }
            decl.push_str("  }");
        }
    } else if !props_macro.args.is_empty() {
        if let (true, Some(destructure)) =
            (needs_merge_defaults, ctx.macros.props_destructure.as_ref())
        {
            decl.push_str("/*@__PURE__*/_mergeDefaults(");
            decl.push_str(&props_macro.args);
            decl.push_str(", {\n");
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
                decl.push_str("  ");
                decl.push_str(&runtime_prop_key(key));
                decl.push_str(": ");
                if binding.default_needs_factory {
                    // Wrap non-literal expressions in a factory: `() => (expr)`.
                    decl.push_str("() => (");
                    decl.push_str(default_val.trim());
                    decl.push(')');
                } else {
                    // Literals, bare identifiers and functions are emitted as-is.
                    decl.push_str(default_val.trim());
                }
                if binding.default_skip_factory {
                    decl.push_str(", __skip_");
                    decl.push_str(key);
                    decl.push_str(": true");
                }
                if i < defaults.len() - 1 {
                    decl.push(',');
                }
                decl.push('\n');
            }
            decl.push_str("})");
        } else {
            decl.push_str(&props_macro.args);
        }
    } else {
        return None;
    }

    if let Some(runtime_defaults) =
        with_defaults_values.and_then(|defaults| defaults.runtime_expression.as_deref())
    {
        let mut merged = TextBuf::with_capacity(decl.len() + runtime_defaults.len() + 38);
        merged.push_str("/*@__PURE__*/_mergeDefaults(");
        merged.push_str(&decl);
        merged.push_str(", ");
        merged.push_str(runtime_defaults);
        merged.push(')');
        decl = merged;
    }

    Some(decl.into_compact())
}

fn should_preserve_prod_prop_type(runtime_js_type: &str) -> bool {
    runtime_js_type.contains("Boolean") || runtime_js_type.contains("Function")
}

fn build_unknown_type_destructured_props_decl(
    decl: &mut TextBuf,
    destructure: &PropsDestructuredBindings,
) {
    decl.push_str("{\n");
    for (i, key) in destructure.keys.iter().enumerate() {
        decl.push_str("    ");
        decl.push_str(key);
        if let Some(binding) = destructure.bindings.get(key.as_str())
            && let Some(default_val) = binding.default.as_ref()
        {
            decl.push_str(": { default: ");
            let default_val = normalize_destructure_default_value_with_factory(
                default_val,
                binding.default_needs_factory,
            );
            decl.push_str(&default_val);
            decl.push_str(" }");
        } else {
            decl.push_str(": {}");
        }
        if i < destructure.keys.len() - 1 {
            decl.push(',');
        }
        decl.push('\n');
    }
    decl.push_str("  }");
}

#[cfg(test)]
mod tests {
    use super::should_preserve_prod_prop_type;

    #[test]
    fn preserves_listener_relevant_prop_types_in_prod() {
        assert!(should_preserve_prod_prop_type("Boolean"));
        assert!(should_preserve_prod_prop_type("Function"));
        assert!(should_preserve_prod_prop_type("[Function, Array]"));
        assert!(!should_preserve_prod_prop_type("String"));
        assert!(!should_preserve_prod_prop_type("Array"));
    }
}
