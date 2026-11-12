//! Types reached through a destructuring pattern: a property of an object
//! type (`getTypeOfSymbol(getPropertyOfType(type, name))`) or an element of a
//! tuple type.
use super::super::checker_memo::CheckerMemo;
use super::details::{DetailCache, type_detail};
use corsa::api::TypeResponse;
use serde_json::Value;
use vize_l0::String as CompactString;

/// One step from a type to a member type.
#[derive(Clone, Debug)]
pub(in crate::linter) enum PathStep {
    /// A named property.
    Property(CompactString),
    /// A tuple element.
    Element(usize),
}

/// The detail of the type `path` leads to from `ty`.
pub(super) async fn detail_at(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    path: &[PathStep],
    cache: &mut DetailCache,
) -> corsa::Result<Option<Value>> {
    let mut current = ty.clone();
    for step in path {
        let next = match step {
            PathStep::Property(name) => property_type(session, &current, name).await,
            PathStep::Element(index) => element_type(session, &current, *index).await,
        };
        let Some(next) = next else {
            return Ok(None);
        };
        current = next;
    }
    Ok(Some(type_detail(session, &current, cache).await?))
}

async fn property_type(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    name: &str,
) -> Option<TypeResponse> {
    let symbol = session
        .client()
        .get_property_of_type(
            session.snapshot().handle.clone(),
            session.project_handle(),
            ty.id.clone(),
            name,
        )
        .await
        .ok()
        .flatten()?;
    session
        .client()
        .get_type_of_symbol(
            session.snapshot().handle.clone(),
            session.project_handle(),
            symbol.id,
        )
        .await
        .ok()
        .flatten()
}

async fn element_type(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    index: usize,
) -> Option<TypeResponse> {
    let is_tuple = session
        .client()
        .is_tuple_type(
            session.snapshot().handle.clone(),
            session.project_handle(),
            ty.id.clone(),
        )
        .await
        .unwrap_or(false);
    if !is_tuple {
        return None;
    }
    session
        .client()
        .get_type_arguments(
            session.snapshot().handle.clone(),
            session.project_handle(),
            ty.id.clone(),
        )
        .await
        .ok()?
        .into_iter()
        .nth(index)
}
