//! `isTypeUnchanged` from typescript-eslint's `no-unnecessary-type-assertion`.
//!
//! Every branch that cannot be decided with the available checker queries
//! answers "changed", so an assertion is never reported on a guess.
use super::Literal;
use super::checker::{CALL, CONSTRUCT, Checker, flags};
use corsa::api::{TypeHandle, TypeResponse};
use vize_l0::{FxHashSet, String as CompactString};

/// Types visited by one `typeContains` walk before it gives up.
const VISIT_BUDGET: usize = 48;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Found {
    Yes,
    No,
    Unknown,
}

pub(super) async fn is_type_unchanged(
    checker: &mut Checker<'_>,
    intersection_annotation: bool,
    literal: Literal,
    uncast: &TypeResponse,
    cast: &TypeResponse,
) -> bool {
    if uncast.id == cast.id {
        return true;
    }
    // Every remaining branch except `T & {}` only rejects, and the verdict
    // needs mutual assignability, so most assertions settle after one or two
    // checker calls instead of walking both types.
    let widening_to_intersection =
        cast.flags & flags::INTERSECTION != 0 && uncast.flags & flags::INTERSECTION == 0;
    if !widening_to_intersection
        && !(checker.assignable(uncast, cast).await && checker.assignable(cast, uncast).await)
    {
        return false;
    }
    let cast_variable = contains(checker, cast, flags::TYPE_VARIABLE).await;
    if intersection_annotation && cast_variable != Found::No {
        return false;
    }
    let uncast_flags = checker.union_flags(uncast).await;
    let cast_flags = checker.union_flags(cast).await;
    if uncast_flags & flags::NON_PRIMITIVE != 0 && cast_flags & flags::NON_PRIMITIVE == 0 {
        return false;
    }
    if has_index_signature(checker, uncast).await != has_index_signature(checker, cast).await {
        return false;
    }
    if contains(checker, uncast, flags::ANY).await != Found::No
        || contains(checker, cast, flags::ANY).await != Found::No
    {
        return false;
    }
    let uncast_variable = contains(checker, uncast, flags::TYPE_VARIABLE).await;
    if cast_variable == Found::Unknown || uncast_variable == Found::Unknown {
        return false;
    }
    if cast_variable == Found::Yes && uncast_variable == Found::No {
        return false;
    }
    match literal {
        Literal::No => {}
        Literal::Other | Literal::Object { empty: true } => return false,
        Literal::Object { empty: false } => {
            for property in checker.properties(cast).await {
                let Some(ty) = checker.type_of_symbol(&property).await else {
                    return false;
                };
                if ty.flags & flags::LITERAL != 0 {
                    return false;
                }
            }
        }
    }
    if cast.flags & flags::INTERSECTION != 0 && uncast.flags & flags::INTERSECTION == 0 {
        return intersection_with_empty_object(checker, uncast, cast).await;
    }
    same_properties(checker, uncast, cast).await && same_type_arguments(checker, uncast, cast).await
}

/// `T & {}` over a type parameter whose constraint is not nullable.
async fn intersection_with_empty_object(
    checker: &mut Checker<'_>,
    uncast: &TypeResponse,
    cast: &TypeResponse,
) -> bool {
    if uncast.flags & flags::TYPE_PARAMETER == 0 {
        return false;
    }
    let parts = checker.parts(cast).await;
    if parts.len() != 2 || !parts.iter().any(|part| part.id == uncast.id) {
        return false;
    }
    let Some(other) = parts.iter().find(|part| part.id != uncast.id).cloned() else {
        return false;
    };
    if !is_empty_object_type(checker, &other).await
        || contains(checker, &other, flags::TYPE_VARIABLE).await != Found::No
    {
        return false;
    }
    let Some(constraint) = checker.base_constraint(uncast).await else {
        return false;
    };
    checker.union_flags(&constraint).await & flags::NULLABLE == 0
}

pub(super) async fn is_empty_object_type(checker: &mut Checker<'_>, ty: &TypeResponse) -> bool {
    if checker.union_flags(ty).await & flags::NON_PRIMITIVE != 0 {
        return true;
    }
    checker.properties(ty).await.is_empty()
        && checker.signatures(ty, CALL).await.is_empty()
        && checker.signatures(ty, CONSTRUCT).await.is_empty()
        && checker.index_info_count(ty).await == 0
}

async fn has_index_signature(checker: &mut Checker<'_>, ty: &TypeResponse) -> bool {
    for part in checker.union_parts(ty).await {
        if checker.index_info_count(&part).await > 0 {
            return true;
        }
    }
    false
}

/// Same property names, and no `readonly` difference the checker could hide.
async fn same_properties(
    checker: &mut Checker<'_>,
    uncast: &TypeResponse,
    cast: &TypeResponse,
) -> bool {
    let uncast_properties = checker.properties(uncast).await;
    let cast_properties = checker.properties(cast).await;
    if uncast_properties.len() != cast_properties.len() {
        return false;
    }
    let names = cast_properties
        .iter()
        .map(|property| property.name.as_str())
        .collect::<FxHashSet<_>>();
    if !uncast_properties
        .iter()
        .all(|property| names.contains(property.name.as_str()))
    {
        return false;
    }
    let shared = uncast_properties.iter().all(|property| {
        cast_properties
            .iter()
            .any(|other| other.name == property.name && other.id == property.id)
    });
    if shared {
        return true;
    }
    // Readonly modifiers are not visible through the checker API, so distinct
    // property symbols only count as equal when neither type mentions one.
    !mentions_readonly(checker, uncast).await && !mentions_readonly(checker, cast).await
}

async fn mentions_readonly(checker: &mut Checker<'_>, ty: &TypeResponse) -> bool {
    let texts: Vec<CompactString> = checker
        .session()
        .render_type_texts(ty)
        .await
        .unwrap_or_default();
    texts.is_empty() || texts.iter().any(|text| text.contains("readonly"))
}

async fn same_type_arguments(
    checker: &mut Checker<'_>,
    uncast: &TypeResponse,
    cast: &TypeResponse,
) -> bool {
    let uncast_arguments = checker.type_arguments(uncast).await;
    let cast_arguments = checker.type_arguments(cast).await;
    uncast_arguments.len() == cast_arguments.len()
        && uncast_arguments
            .iter()
            .zip(&cast_arguments)
            .all(|(left, right)| left.id == right.id)
}

/// `typeContains(type, t => isTypeFlagSet(t, mask))`, walking union and
/// intersection parts, type arguments and call signatures.
pub(super) async fn contains(checker: &mut Checker<'_>, ty: &TypeResponse, mask: u32) -> Found {
    let mut seen = FxHashSet::<TypeHandle>::default();
    let mut pending = vec![ty.clone()];
    while let Some(current) = pending.pop() {
        if !seen.insert(current.id.clone()) {
            continue;
        }
        if seen.len() > VISIT_BUDGET {
            return Found::Unknown;
        }
        if current.flags & mask != 0 {
            return Found::Yes;
        }
        if current.flags & (flags::UNION | flags::INTERSECTION) != 0 {
            pending.extend(checker.parts(&current).await);
            continue;
        }
        pending.extend(checker.type_arguments(&current).await);
        if current.flags & flags::OBJECT != 0 {
            for signature in checker.signatures(&current, CALL).await {
                if let Some(returned) = checker.return_type(&signature).await {
                    pending.push(returned);
                }
                pending.extend(checker.parameter_types(&signature).await);
            }
        }
    }
    Found::No
}
