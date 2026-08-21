//! Indexed access prop runtime-type resolution.

use oxc_ast::ast::TSIndexedAccessType;
use vize_carton::FxHashMap;
use vize_carton::String;

use super::ast_resolve::{
    collect_props_from_ts_type, combine_runtime_js_types, literal_values_from_ts_type,
};

pub(super) fn is_utility_object_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "Partial" | "Required" | "Pick" | "Omit" | "Record"
    )
}

pub(super) fn resolve_indexed_access_js_type(
    indexed: &TSIndexedAccessType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> Option<String> {
    let keys = literal_values_from_ts_type(
        &indexed.index_type,
        source,
        interfaces,
        type_aliases,
        None,
        seen,
    )?;
    let mut props = Vec::new();
    if !collect_props_from_ts_type(
        &indexed.object_type,
        source,
        interfaces,
        type_aliases,
        seen,
        &mut props,
    ) {
        return None;
    }

    let mut js_types = Vec::new();
    for key in keys {
        js_types.extend(
            props
                .iter()
                .filter_map(|(name, info)| (name == key).then_some(info.js_type.clone())),
        );
    }
    (!js_types.is_empty()).then(|| combine_runtime_js_types(js_types))
}
