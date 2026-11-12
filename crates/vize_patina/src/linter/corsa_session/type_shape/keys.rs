//! Per-key property facts for computed member access (`obj[key]`).
use super::super::checker_memo::CheckerMemo;
use super::{
    INTERSECTION, JsonMap, NUMBER_LITERAL, STRING_LIKE, STRING_LITERAL, ShapeCache, TYPE_PARAMETER,
    UNION, constituents, constraint, property,
};
use corsa::api::{IndexInfo, TypeResponse};
use serde_json::{Map, Value};

/// One constituent of a computed member key type.
pub(super) struct Key {
    /// String or number literal key, as the property name it reads.
    pub(super) name: Option<vize_l0::String>,
    /// `{ key?, typeName }` for the shape.
    pub(super) json: Value,
}

/// typescript-eslint `getTypeName` for a key type: `"string"` for string-like
/// types (and unions of them, intersections with one, and type parameters
/// constrained to one), else the checker's rendering.
pub(super) async fn type_name(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut ShapeCache,
) -> corsa::Result<vize_l0::String> {
    if is_string_named(session, ty, cache, 0).await? {
        return Ok(vize_l0::String::from("string"));
    }
    Ok(session
        .client()
        .type_to_string(
            session.snapshot().handle.clone(),
            session.project_handle(),
            ty.id.clone(),
            None,
            None,
        )
        .await
        .map(vize_l0::String::from)
        .unwrap_or_default())
}

pub(super) async fn is_string_named(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut ShapeCache,
    depth: u8,
) -> corsa::Result<bool> {
    if ty.flags & STRING_LIKE != 0 {
        return Ok(true);
    }
    if depth > 4 {
        return Ok(false);
    }
    if ty.flags & TYPE_PARAMETER != 0 {
        return Ok(constraint(session, ty, cache)
            .await?
            .is_some_and(|constrained| constrained.flags & STRING_LIKE != 0));
    }
    if ty.flags & (UNION | INTERSECTION) == 0 {
        return Ok(false);
    }
    let parts = constituents(session, ty, cache).await?;
    let mut all = !parts.is_empty();
    let mut some = false;
    for part in &parts {
        let named = Box::pin(is_string_named(session, part, cache, depth + 1)).await?;
        all &= named;
        some |= named;
    }
    Ok(if ty.flags & UNION != 0 { all } else { some })
}

/// The constituents of a computed key's constrained type.
pub(super) async fn keys(
    session: &CheckerMemo<'_>,
    key: &TypeResponse,
    cache: &mut ShapeCache,
) -> corsa::Result<Vec<Key>> {
    let key = match constraint(session, key, cache).await? {
        Some(constrained) => constrained,
        None => key.clone(),
    };
    let parts = if key.flags & UNION != 0 {
        constituents(session, &key, cache).await?
    } else {
        vec![key]
    };
    let mut keys = Vec::with_capacity(parts.len());
    for part in parts {
        let name = if part.flags & (STRING_LITERAL | NUMBER_LITERAL) != 0 {
            part.value.as_ref().and_then(property_name)
        } else {
            None
        };
        let mut json = Map::new();
        if let Some(name) = &name {
            json.insert("key".into(), Value::from(name.as_str()));
        }
        json.insert(
            "typeName".into(),
            Value::from(type_name(session, &part, cache).await?.as_str()),
        );
        keys.push(Key {
            name,
            json: Value::Object(json),
        });
    }
    Ok(keys)
}

/// `String(value)` of a string or number literal value.
pub(super) fn property_name(value: &Value) -> Option<vize_l0::String> {
    match value {
        Value::String(text) => Some(vize_l0::String::from(text.as_str())),
        Value::Number(number) => Some(match number.as_i64() {
            Some(integer) => vize_l0::cstr!("{integer}"),
            None => vize_l0::cstr!("{number}"),
        }),
        _ => None,
    }
}

/// `keyed`: property facts per literal key (null for other keys), and
/// `indexKeys`: the type names of the index signatures' key types.
pub(super) async fn keyed_facts(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    keys: &[Key],
    entry: &mut JsonMap,
    cache: &mut ShapeCache,
) -> corsa::Result<()> {
    let mut keyed = Vec::with_capacity(keys.len());
    for key in keys {
        keyed.push(match &key.name {
            Some(name) => property(session, ty, name, cache).await?,
            None => Value::Null,
        });
    }
    entry.insert("keyed".into(), Value::Array(keyed));
    let infos: Vec<IndexInfo> = session
        .client()
        .get_index_infos_of_type(
            session.snapshot().handle.clone(),
            session.project_handle(),
            ty.id.clone(),
        )
        .await
        .unwrap_or_default();
    let mut names = Vec::with_capacity(infos.len());
    for info in infos {
        names.push(Value::from(
            type_name(session, &info.key_type, cache).await?.as_str(),
        ));
    }
    entry.insert("indexKeys".into(), Value::Array(names));
    Ok(())
}
