use oxc_ast::ast::TSIndexedAccessType;
use vize_carton::{FxHashMap, String};

use super::runtime_type_combine::combine_runtime_js_types;
use super::{collect_props_from_ts_type, literal_values_from_ts_type};

pub(super) fn resolve(
    indexed: &TSIndexedAccessType<'_>,
    source: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
    seen: &mut Vec<String>,
) -> String {
    resolve_inner(indexed, source, interfaces, type_aliases, seen).unwrap_or_else(|| "null".into())
}

fn resolve_inner(
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
        let (_, prop) = props
            .iter()
            .find(|(name, _)| name.as_str() == key.as_str())?;
        js_types.push(prop.js_type.clone());
    }

    Some(combine_runtime_js_types(js_types))
}
