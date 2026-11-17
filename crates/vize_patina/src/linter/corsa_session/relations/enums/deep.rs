//! Recursive enum identity and structural mismatch checks.
use super::super::checker::{Checker, flags};
use corsa::api::{TypeHandle, TypeResponse};
use std::future::Future;
use std::pin::Pin;
use vize_l0::{FxHashSet, String as CompactString};

type Boxed<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

const ENUM_LITERAL: u32 = 1 << 15;
const STRING_LITERAL: u32 = 1 << 10;
const NUMBER_LITERAL: u32 = 1 << 11;
const REFERENCE: u32 = 1 << 2;
const MAX_DEPTH: u8 = 5;

pub(super) async fn mismatch(
    checker: &mut Checker<'_>,
    sender: &TypeResponse,
    receiver: &TypeResponse,
) -> bool {
    let key = (sender.id.clone(), receiver.id.clone());
    if let Some(known) = checker.enum_mismatches.get(&key) {
        return *known;
    }
    let mut visited = FxHashSet::default();
    let found = deep_mismatch(
        checker,
        sender.clone(),
        receiver.clone(),
        &mut visited,
        0,
        false,
    )
    .await;
    checker.enum_mismatches.insert(key, found);
    found
}

fn deep_mismatch<'a>(
    checker: &'a mut Checker<'_>,
    sender: TypeResponse,
    receiver: TypeResponse,
    visited: &'a mut FxHashSet<(TypeHandle, TypeHandle)>,
    depth: u8,
    within_generic_members: bool,
) -> Boxed<'a, bool> {
    Box::pin(async move {
        if depth > MAX_DEPTH || sender.id == receiver.id {
            return false;
        }
        let receiver = constrained(checker, receiver).await;
        if !may_hold_enum(&receiver) {
            return false;
        }
        let sender = constrained(checker, sender).await;
        if sender.id == receiver.id {
            return false;
        }
        if !visited.insert((sender.id.clone(), receiver.id.clone())) {
            return false;
        }
        if direct_mismatch(checker, &sender, &receiver).await {
            return true;
        }
        // Each pair below is compared only when the receiver has that part,
        // so the sender's side is not asked for in vain.
        let receiver_arguments = reference_arguments(checker, &receiver).await;
        let sender_arguments = if receiver_arguments.is_empty() {
            Vec::new()
        } else {
            reference_arguments(checker, &sender).await
        };
        if sender_arguments.len() == receiver_arguments.len() {
            for (sender, receiver) in sender_arguments.into_iter().zip(receiver_arguments) {
                if deep_mismatch(
                    checker,
                    sender,
                    receiver,
                    visited,
                    depth + 1,
                    within_generic_members,
                )
                .await
                {
                    return true;
                }
            }
        }
        let generic =
            generic_type(checker, &sender).await || generic_type(checker, &receiver).await;
        if generic && within_generic_members {
            return false;
        }
        if let Some(receiver) = number_index(checker, &receiver).await
            && may_hold_enum(&receiver)
            && let Some(sender) = number_index(checker, &sender).await
            && deep_mismatch(
                checker,
                sender,
                receiver,
                visited,
                depth + 1,
                generic || within_generic_members,
            )
            .await
        {
            return true;
        }
        let receiver_parts = enum_parts(checker, &receiver).await;
        if !receiver_parts.is_empty()
            && receiver_parts
                .iter()
                .all(|part| part.flags & ENUM_LITERAL != 0)
        {
            return false;
        }
        let receiver = checker.apparent(&receiver).await;
        let receiver_properties = checker.properties(&receiver).await;
        if receiver_properties.is_empty() {
            return false;
        }
        // The sender's members are read once a receiver member can hold an
        // enum at all.
        let mut sender_properties = None;
        for receiver_property in receiver_properties {
            let Some(receiver) = checker.type_of_symbol(&receiver_property).await else {
                continue;
            };
            if !may_hold_enum(&receiver) {
                continue;
            }
            if sender_properties.is_none() {
                let apparent = checker.apparent(&sender).await;
                sender_properties = Some(checker.properties(&apparent).await);
            }
            let Some(sender_property) = sender_properties
                .iter()
                .flatten()
                .find(|property| property.name == receiver_property.name)
            else {
                continue;
            };
            if sender_property.id == receiver_property.id {
                continue;
            }
            let Some(sender) = checker.type_of_symbol(sender_property).await else {
                continue;
            };
            if deep_mismatch(
                checker,
                sender,
                receiver,
                visited,
                depth + 1,
                generic || within_generic_members,
            )
            .await
            {
                return true;
            }
        }
        false
    })
}

/// `getBaseConstraintOfType`, asked only of the types that can have a
/// constraint other than themselves (type variables and compositions).
async fn constrained(checker: &mut Checker<'_>, ty: TypeResponse) -> TypeResponse {
    if ty.flags & (flags::INSTANTIABLE | flags::UNION) == 0 {
        return ty;
    }
    checker.base_constraint(&ty).await.unwrap_or(ty)
}

/// Whether a receiver can hold an enum anywhere the relation reads: an enum
/// or enum literal, a union, an object (members, type arguments, number
/// index) or a type whose constraint may. Primitives, non-enum literals,
/// `any`, `unknown`, `never` and `object` hold none, so no value assigned to
/// them can be an unsafe enum flow.
fn may_hold_enum(ty: &TypeResponse) -> bool {
    ty.flags & (flags::ENUM_LIKE | flags::UNION | flags::OBJECT | flags::INSTANTIABLE) != 0
}

async fn direct_mismatch(
    checker: &mut Checker<'_>,
    sender: &TypeResponse,
    receiver: &TypeResponse,
) -> bool {
    let receiver_parts = enum_parts(checker, receiver).await;
    let mut receiver_enums = Vec::new();
    let mut number_enum = false;
    let mut string_enum = false;
    let mut non_enum = Vec::new();
    for part in receiver_parts {
        if part.flags & flags::ENUM != 0 {
            number_enum = true;
            string_enum = true;
            if !receiver_enums
                .iter()
                .any(|known: &TypeResponse| known.id == part.id)
            {
                receiver_enums.push(part);
            }
            continue;
        }
        if part.flags & ENUM_LITERAL == 0 {
            non_enum.push(part);
            continue;
        }
        if part.flags & NUMBER_LITERAL != 0 {
            number_enum = true;
        } else {
            string_enum = true;
        }
        if let Some(base) = enum_base(checker, &part).await
            && !receiver_enums
                .iter()
                .any(|known: &TypeResponse| known.id == base.id)
        {
            receiver_enums.push(base);
        }
    }
    if receiver_enums.is_empty() {
        return false;
    }
    for part in enum_parts(checker, sender).await {
        let kind_matches = (number_enum && part.flags & flags::NUMBER_LIKE != 0)
            || (string_enum && part.flags & (flags::STRING | STRING_LITERAL) != 0);
        if !kind_matches {
            continue;
        }
        if let Some(base) = enum_base(checker, &part).await
            && receiver_enums.iter().any(|expected| expected.id == base.id)
        {
            continue;
        }
        let mut accepted = false;
        for receiver in &non_enum {
            if checker.assignable(&part, receiver).await {
                accepted = true;
                break;
            }
        }
        if !accepted {
            return true;
        }
    }
    false
}

async fn enum_base(checker: &mut Checker<'_>, ty: &TypeResponse) -> Option<TypeResponse> {
    if ty.flags & flags::ENUM != 0 {
        return Some(ty.clone());
    }
    if ty.flags & ENUM_LITERAL == 0 {
        return None;
    }
    checker
        .session()
        .client()
        .get_base_type_of_literal_type(
            checker.session().snapshot().handle.clone(),
            checker.session().project_handle(),
            ty.id.clone(),
        )
        .await
        .ok()
        .flatten()
}

async fn enum_parts(checker: &mut Checker<'_>, ty: &TypeResponse) -> Vec<TypeResponse> {
    let parts = checker.union_parts(ty).await;
    if parts.len() != 1 || parts.first().is_none_or(|part| part.id != ty.id) {
        return parts;
    }
    if ty.flags & flags::ENUM == 0 {
        return parts;
    }
    if let Some(expanded) = checker.enum_constituents.get(&ty.id) {
        return expanded.clone();
    }
    let expanded = checker
        .session()
        .client()
        .get_types_of_type_in_project(
            checker.session().snapshot().handle.clone(),
            checker.session().project_handle(),
            ty.id.clone(),
        )
        .await
        .unwrap_or_default();
    if expanded.is_empty() {
        return parts;
    }
    checker
        .enum_constituents
        .insert(ty.id.clone(), expanded.clone());
    expanded
}

fn generic_type<'a>(checker: &'a mut Checker<'_>, ty: &'a TypeResponse) -> Boxed<'a, bool> {
    Box::pin(async move {
        if let Some(known) = checker.generic_types.get(&ty.id) {
            return *known;
        }
        if ty.flags & flags::INSTANTIABLE != 0 {
            checker.generic_types.insert(ty.id.clone(), true);
            return true;
        }
        // Break recursive generic definitions pessimistically, then replace
        // the entry once their parts and arguments are known.
        checker.generic_types.insert(ty.id.clone(), false);
        let mut children = checker.parts(ty).await;
        children.extend(checker.type_arguments(ty).await);
        for child in &children {
            if generic_type(checker, child).await {
                checker.generic_types.insert(ty.id.clone(), true);
                return true;
            }
        }
        false
    })
}

pub(super) async fn reference_arguments(
    checker: &mut Checker<'_>,
    ty: &TypeResponse,
) -> Vec<TypeResponse> {
    if ty.object_flags.unwrap_or(0) & REFERENCE == 0 {
        return Vec::new();
    }
    checker
        .session()
        .client()
        .get_type_arguments(
            checker.session().snapshot().handle.clone(),
            checker.session().project_handle(),
            ty.id.clone(),
        )
        .await
        .unwrap_or_default()
}

pub(super) async fn number_index(
    checker: &mut Checker<'_>,
    ty: &TypeResponse,
) -> Option<TypeResponse> {
    checker
        .session()
        .client()
        .get_index_infos_of_type(
            checker.session().snapshot().handle.clone(),
            checker.session().project_handle(),
            ty.id.clone(),
        )
        .await
        .unwrap_or_default()
        .into_iter()
        .find(|info| info.key_type.flags & flags::NUMBER != 0)
        .map(|info| info.value_type)
}

pub(super) async fn enum_names(checker: &mut Checker<'_>, ty: &TypeResponse) -> Vec<CompactString> {
    if let Some(known) = checker.enum_names.get(&ty.id) {
        return known.clone();
    }
    let mut visited = FxHashSet::default();
    let mut names = Vec::new();
    collect_enum_names(checker, ty.clone(), &mut visited, &mut names).await;
    names.sort_unstable();
    names.dedup();
    checker.enum_names.insert(ty.id.clone(), names.clone());
    names
}

fn collect_enum_names<'a>(
    checker: &'a mut Checker<'_>,
    ty: TypeResponse,
    visited: &'a mut FxHashSet<TypeHandle>,
    names: &'a mut Vec<CompactString>,
) -> Boxed<'a, ()> {
    // typescript-eslint `describeEnumTypes` walks without a depth bound; the
    // visited set ends recursive types. A bound here could hide the enum of
    // a member the bounded comparison reached by a shorter path.
    Box::pin(async move {
        let ty = checker.base_constraint(&ty).await.unwrap_or(ty);
        if !visited.insert(ty.id.clone()) {
            return;
        }
        for part in enum_parts(checker, &ty).await {
            if part.flags & flags::ENUM != 0 {
                names.extend(checker.type_texts(&part).await);
            } else if let Some(base) = enum_base(checker, &part).await {
                names.extend(checker.type_texts(&base).await);
            }
        }
        for argument in reference_arguments(checker, &ty).await {
            collect_enum_names(checker, argument, visited, names).await;
        }
        if let Some(element) = number_index(checker, &ty).await {
            collect_enum_names(checker, element, visited, names).await;
        }
        for property in checker.properties(&ty).await {
            if let Some(property) = checker.type_of_symbol(&property).await {
                collect_enum_names(checker, property, visited, names).await;
            }
        }
    })
}
