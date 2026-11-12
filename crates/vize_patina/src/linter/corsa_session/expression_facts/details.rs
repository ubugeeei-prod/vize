//! Checker type details typescript-eslint reads off `ts.Type` rather than its
//! rendered text: flags, union constituents, the enum an enum literal belongs
//! to, and the base constraint of a generic type.
use super::super::checker_memo::CheckerMemo;
use corsa::api::{TypeHandle, TypeResponse};
use serde_json::{Value, json};
use vize_l0::FxHashMap;

// TypeScript 7 TypeFlags, from microsoft/typescript-go internal/checker/types.go.
const ENUM_LITERAL: u32 = 1 << 15;
const TYPE_PARAMETER: u32 = 1 << 19;
const INDEX: u32 = 1 << 21;
const SUBSTITUTION: u32 = 1 << 24;
const INDEXED_ACCESS: u32 = 1 << 25;
const CONDITIONAL: u32 = 1 << 26;
const UNION: u32 = 1 << 27;
const INTERSECTION: u32 = 1 << 28;
/// Types `getBaseConstraintOfType` can answer with something other than itself.
const CONSTRAINABLE: u32 =
    TYPE_PARAMETER | INDEX | SUBSTITUTION | INDEXED_ACCESS | CONDITIONAL | UNION | INTERSECTION;
/// TypeScript 7 `ObjectFlags.Reference`: a generic instantiation (including
/// arrays and tuples) whose `target` and type arguments are meaningful.
const REFERENCE: u32 = 1 << 2;
/// How deep type arguments of type references are described.
const MAX_ARGUMENT_DEPTH: u8 = 4;

/// Per-snapshot cache of rendered details, keyed by type handle.
#[derive(Default)]
pub(super) struct DetailCache {
    details: FxHashMap<TypeHandle, Value>,
}

/// `{ id, flags, objectFlags?, target?, typeArguments?, parts, constraint? }`
/// for one checker type.
///
/// `parts` are the union constituents (the type itself when it is not a union).
/// Each part carries its own `flags`, `memberFlags` (the flags of every
/// intersection constituent OR-ed together, which is what `isNumberLike` and
/// friends test) and, for an enum literal, the `enumId` of its enum type.
pub(super) async fn type_detail(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut DetailCache,
) -> corsa::Result<Value> {
    detail_at_depth(session, ty, cache, 0).await
}

async fn detail_at_depth(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut DetailCache,
    depth: u8,
) -> corsa::Result<Value> {
    if let Some(detail) = cache.details.get(&ty.id) {
        return Ok(detail.clone());
    }
    let mut detail = json!({
        "id": ty.id.as_str(),
        "flags": ty.flags,
        "parts": parts(session, ty).await?,
    });
    if let Some(object_flags) = ty.object_flags {
        if let Some(object) = detail.as_object_mut() {
            object.insert("objectFlags".into(), json!(object_flags));
            if let Some(target) = &ty.target {
                object.insert("target".into(), json!(target.as_str()));
            }
        }
        if object_flags & REFERENCE != 0 && depth < MAX_ARGUMENT_DEPTH {
            let arguments = session
                .client()
                .get_type_arguments(
                    session.snapshot().handle.clone(),
                    session.project_handle(),
                    ty.id.clone(),
                )
                .await
                .unwrap_or_default();
            let mut described = Vec::with_capacity(arguments.len());
            for argument in &arguments {
                described
                    .push(Box::pin(detail_at_depth(session, argument, cache, depth + 1)).await?);
            }
            let is_array = is_array_type(session, ty).await;
            let is_tuple = is_tuple_type(session, ty).await;
            let text = session
                .render_type_texts(ty)
                .await
                .ok()
                .and_then(|texts| texts.into_iter().next());
            if let (Some(text), Some(object)) = (text, detail.as_object_mut()) {
                object.insert("text".into(), json!(text.as_str()));
            }
            if let Some(object) = detail.as_object_mut() {
                object.insert("typeArguments".into(), Value::Array(described));
                object.insert("isArray".into(), json!(is_array));
                object.insert("isTuple".into(), json!(is_tuple));
            }
        }
    }
    if ty.flags & CONSTRAINABLE != 0 {
        let constraint = session.base_constraint(ty).await;
        if let Some(constraint) = constraint.filter(|constraint| constraint.id != ty.id) {
            let parts = parts(session, &constraint).await?;
            if let Some(object) = detail.as_object_mut() {
                object.insert(
                    "constraint".into(),
                    json!({
                        "id": constraint.id.as_str(),
                        "flags": constraint.flags,
                        "parts": parts,
                    }),
                );
            }
        }
    }
    cache.details.insert(ty.id.clone(), detail.clone());
    Ok(detail)
}

async fn parts(session: &CheckerMemo<'_>, ty: &TypeResponse) -> corsa::Result<Vec<Value>> {
    let members = if ty.flags & UNION != 0 {
        constituents(session, ty).await
    } else {
        Vec::new()
    };
    let members = if members.is_empty() {
        vec![ty.clone()]
    } else {
        members
    };
    let mut parts = Vec::with_capacity(members.len());
    for member in &members {
        parts.push(part(session, member).await?);
    }
    Ok(parts)
}

async fn part(session: &CheckerMemo<'_>, ty: &TypeResponse) -> corsa::Result<Value> {
    let member_flags = if ty.flags & INTERSECTION != 0 {
        constituents(session, ty)
            .await
            .iter()
            .fold(0, |flags, member| flags | member.flags)
    } else {
        ty.flags
    };
    let mut part = json!({
        "id": ty.id.as_str(),
        "flags": ty.flags,
        "memberFlags": member_flags,
    });
    if ty.flags & ENUM_LITERAL != 0 {
        let base = session
            .client()
            .get_base_type_of_literal_type(
                session.snapshot().handle.clone(),
                session.project_handle(),
                ty.id.clone(),
            )
            .await
            .ok()
            .flatten();
        if let (Some(base), Some(object)) = (base, part.as_object_mut()) {
            object.insert("enumId".into(), json!(base.id.as_str()));
            let name = session
                .render_type_texts(&base)
                .await
                .ok()
                .and_then(|texts| texts.into_iter().next());
            if let Some(name) = name {
                object.insert("enumText".into(), json!(name.as_str()));
            }
        }
    }
    Ok(part)
}

async fn constituents(session: &CheckerMemo<'_>, ty: &TypeResponse) -> Vec<TypeResponse> {
    session.constituents(ty).await
}

async fn is_array_type(session: &CheckerMemo<'_>, ty: &TypeResponse) -> bool {
    session
        .client()
        .is_array_type(
            session.snapshot().handle.clone(),
            session.project_handle(),
            ty.id.clone(),
        )
        .await
        .unwrap_or(false)
}

async fn is_tuple_type(session: &CheckerMemo<'_>, ty: &TypeResponse) -> bool {
    session
        .client()
        .is_tuple_type(
            session.snapshot().handle.clone(),
            session.project_handle(),
            ty.id.clone(),
        )
        .await
        .unwrap_or(false)
}
