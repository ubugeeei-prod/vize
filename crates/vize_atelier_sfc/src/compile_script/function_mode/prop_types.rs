//! Type-based `defineProps` runtime types for function mode.

use vize_carton::String;

use crate::script::{ScriptCompileContext, resolve_type_args};

use super::super::props::{
    PropTypeInfo, extract_prop_types_from_type_with_context, resolve_prop_js_type,
};

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

pub(super) fn resolve_prop_type(prop_type: &PropTypeInfo, ctx: &ScriptCompileContext) -> String {
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
