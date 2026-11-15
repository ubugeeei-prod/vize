//! Deep enum-flow relations used by `no-unsafe-enum-assignment`.
//!
//! The lint listener's compact type facts settle direct enum assignments.
//! Structural members, contextual types, resolved signatures and computed
//! enum keys need paired checker types, so they are resolved here in the same
//! expression batch as the other relation rules.
use super::checker::{CALL, CONSTRUCT, Checker, flags};
use super::{EnumCallAsk, EnumCheckAsk, EnumMismatch, EnumReceiver, Located, Span};
use crate::linter::corsa_session::boolean::checker_node_text;
use corsa::api::{NodeHandle, SignatureResponse, TypeHandle, TypeResponse};
use std::future::Future;
use std::pin::Pin;
use vize_l0::{FxHashSet, String as CompactString};

type Boxed<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

const ENUM_LITERAL: u32 = 1 << 15;
const STRING_LITERAL: u32 = 1 << 10;
const NUMBER_LITERAL: u32 = 1 << 11;
const REFERENCE: u32 = 1 << 2;
const MAX_DEPTH: u8 = 5;

pub(super) async fn check(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &EnumCheckAsk,
    translate: &dyn Fn(Span) -> Span,
) -> Option<EnumMismatch> {
    match ask {
        EnumCheckAsk::Pair {
            sender,
            receiver,
            safe_operands,
        } => {
            let sender = located.ty(translate(*sender))?.clone();
            let receiver = match receiver {
                EnumReceiver::TypeAt(span) => located.ty(translate(*span)).cloned(),
                EnumReceiver::Contextual(span) => {
                    let handle = located.handle(translate(*span))?;
                    checker.contextual(handle).await
                }
            }?;
            if mismatch(checker, &sender, &receiver).await {
                if !safe_operands.is_empty() {
                    let mut safe = true;
                    for operand in safe_operands {
                        let Some(operand) = located.ty(translate(*operand)).cloned() else {
                            safe = false;
                            break;
                        };
                        if mismatch(checker, &operand, &receiver).await {
                            safe = false;
                            break;
                        }
                    }
                    if safe {
                        return None;
                    }
                }
                result(checker, &receiver).await
            } else {
                None
            }
        }
        EnumCheckAsk::Return {
            function,
            returned,
            receiver,
            is_async,
        } => {
            let mut sender = located.ty(translate(*returned))?.clone();
            let explicit = receiver
                .and_then(|receiver| located.ty(translate(receiver)))
                .cloned();
            let contextual = match located.handle(translate(*returned)) {
                Some(handle) => checker.contextual(handle).await,
                None => None,
            };
            let from_context = contextual.is_some();
            let mut receiver = match contextual.or(explicit) {
                Some(receiver) => receiver,
                None => {
                    let own = located.ty(translate(*function))?.clone();
                    let signature = checker.signatures(&own, CALL).await.into_iter().next()?;
                    checker.return_type(&signature).await?
                }
            };
            if !from_context && *is_async {
                receiver = awaited(checker, &receiver).await;
                sender = awaited(checker, &sender).await;
            }
            mismatch(checker, &sender, &receiver).await.then_some(())?;
            result(checker, &receiver).await
        }
        EnumCheckAsk::Heritage {
            sender,
            member,
            sources,
        } => {
            let sender = located.ty(translate(*sender))?.clone();
            let mut receivers = Vec::new();
            for source in sources {
                let Some(ty) = located.ty(translate(source.span)).cloned() else {
                    continue;
                };
                let types = if source.base_types {
                    let constructors = checker.signatures(&ty, CONSTRUCT).await;
                    let instance = match constructors.first() {
                        Some(signature) => checker
                            .return_type(signature)
                            .await
                            .unwrap_or_else(|| ty.clone()),
                        None => ty.clone(),
                    };
                    checker
                        .session()
                        .client()
                        .get_base_types(
                            checker.session().snapshot().handle.clone(),
                            checker.session().project_handle(),
                            instance.id,
                        )
                        .await
                        .unwrap_or_default()
                } else {
                    vec![ty]
                };
                for ty in types {
                    let Some(property) = checker.property(&ty, member.as_str()).await else {
                        continue;
                    };
                    if let Some(ty) = checker.type_of_symbol(&property).await {
                        receivers.push(ty);
                    }
                }
            }
            if receivers.is_empty() {
                return None;
            }
            for receiver in &receivers {
                if !mismatch(checker, &sender, receiver).await {
                    return None;
                }
            }
            results(checker, &receivers).await
        }
        EnumCheckAsk::Computed { object, property } => {
            let object = located.ty(translate(*object))?.clone();
            let sender = located.ty(translate(*property))?.clone();
            computed_receivers(checker, &object, &sender).await
        }
    }
}

pub(super) async fn call(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &EnumCallAsk,
    translate: &dyn Fn(Span) -> Span,
) -> Vec<Option<EnumMismatch>> {
    let mut replies = vec![None; ask.arguments.len()];
    let Some(handle) = located.handle(translate(ask.call)).cloned() else {
        return replies;
    };
    let signature = checker.resolved_signature(&handle).await;
    let Some(signature) = signature else {
        return replies;
    };
    let mut position = usize::from(ask.tagged_template);
    let mut remaining = false;
    for (index, argument) in ask.arguments.iter().enumerate() {
        if !argument.spread {
            let Some(sender) = located.ty(translate(argument.expression)).cloned() else {
                position = position.saturating_add(1);
                continue;
            };
            let receiver = argument_receiver(
                checker, located, argument, translate, &signature, position, remaining,
            )
            .await;
            if let Some(receiver) = receiver
                && mismatch(checker, &sender, &receiver).await
            {
                let answer = result(checker, &receiver).await;
                if let Some(reply) = replies.get_mut(index) {
                    *reply = answer;
                }
            }
            position = position.saturating_add(1);
            continue;
        }
        let Some(spread) = located.ty(translate(argument.expression)).cloned() else {
            continue;
        };
        if checker.is_tuple(&spread).await {
            let elements = reference_arguments(checker, &spread).await;
            let mut receivers = Vec::new();
            for element in &elements {
                let receiver = parameter_type(checker, &signature, position, remaining).await;
                if let Some(receiver) = receiver
                    && mismatch(checker, element, &receiver).await
                {
                    receivers.push(receiver);
                }
                position = position.saturating_add(1);
            }
            if !receivers.is_empty() {
                let answer = results(checker, &receivers).await;
                if let Some(reply) = replies.get_mut(index) {
                    *reply = answer;
                }
            }
            if spread
                .element_flags
                .iter()
                .any(|flag| flag & ((1 << 2) | (1 << 3)) != 0)
            {
                remaining = true;
            }
            continue;
        }
        let sender = spread_value(checker, located, argument, translate)
            .await
            .unwrap_or(spread);
        let receiver = argument_receiver(
            checker, located, argument, translate, &signature, position, remaining,
        )
        .await;
        if let Some(receiver) = receiver
            && mismatch(checker, &sender, &receiver).await
        {
            let answer = result(checker, &receiver).await;
            if let Some(reply) = replies.get_mut(index) {
                *reply = answer;
            }
        }
        position = position.saturating_add(1);
    }
    replies
}

async fn parameter_type(
    checker: &mut Checker<'_>,
    signature: &SignatureResponse,
    position: usize,
    remaining: bool,
) -> Option<TypeResponse> {
    if remaining && let Some(receiver) = variadic_receiver(checker, signature).await {
        return Some(receiver);
    }
    checker
        .parameter_type(&signature.id, i32::try_from(position).unwrap_or(i32::MAX))
        .await
}

async fn variadic_receiver(
    checker: &mut Checker<'_>,
    signature: &SignatureResponse,
) -> Option<TypeResponse> {
    if signature.flags & 1 == 0 {
        return None;
    }
    let index = signature.parameters.len().checked_sub(1)?;
    checker
        .parameter_type(&signature.id, i32::try_from(index).ok()?)
        .await
}

async fn argument_receiver(
    checker: &mut Checker<'_>,
    located: &Located,
    argument: &super::EnumCallArgument,
    translate: &dyn Fn(Span) -> Span,
    signature: &SignatureResponse,
    position: usize,
    remaining: bool,
) -> Option<TypeResponse> {
    if let Some(parameter) = parameter_type(checker, signature, position, remaining).await {
        return Some(parameter);
    }
    let handle = located.handle(translate(argument.node))?;
    checker.contextual(handle).await
}

async fn spread_value(
    checker: &mut Checker<'_>,
    located: &Located,
    argument: &super::EnumCallArgument,
    translate: &dyn Fn(Span) -> Span,
) -> Option<TypeResponse> {
    if let Some(node) = located.ty(translate(argument.node)).cloned()
        && located
            .ty(translate(argument.expression))
            .is_none_or(|expression| expression.id != node.id)
    {
        return Some(node);
    }
    let expression = located.ty(translate(argument.expression))?.clone();
    let arguments = reference_arguments(checker, &expression).await;
    if arguments.len() == 1 {
        return arguments.into_iter().next();
    }
    number_index(checker, &expression).await
}

async fn awaited(checker: &mut Checker<'_>, ty: &TypeResponse) -> TypeResponse {
    reference_arguments(checker, ty)
        .await
        .into_iter()
        .next()
        .unwrap_or_else(|| ty.clone())
}

async fn result(checker: &mut Checker<'_>, receiver: &TypeResponse) -> Option<EnumMismatch> {
    results(checker, std::slice::from_ref(receiver)).await
}

async fn results(checker: &mut Checker<'_>, receivers: &[TypeResponse]) -> Option<EnumMismatch> {
    let mut names = Vec::new();
    for receiver in receivers {
        names.extend(enum_names(checker, receiver).await);
    }
    names.sort_unstable();
    names.dedup();
    (!names.is_empty()).then_some(EnumMismatch { names })
}

async fn computed_receivers(
    checker: &mut Checker<'_>,
    object: &TypeResponse,
    sender: &TypeResponse,
) -> Option<EnumMismatch> {
    let object = checker.apparent(object).await;
    let mut receivers = checker
        .session()
        .client()
        .get_index_infos_of_type(
            checker.session().snapshot().handle.clone(),
            checker.session().project_handle(),
            object.id.clone(),
        )
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|info| info.key_type)
        .collect::<Vec<_>>();
    for property in checker.properties(&object).await {
        for declaration in property
            .declarations
            .iter()
            .chain(property.value_declaration.iter())
        {
            if let Some(ty) = computed_declaration_type(checker, declaration).await
                && !receivers.iter().any(|known| known.id == ty.id)
            {
                receivers.push(ty);
            }
        }
    }
    if receivers.is_empty() {
        return None;
    }
    for receiver in &receivers {
        if !mismatch(checker, sender, receiver).await && checker.assignable(sender, receiver).await
        {
            return None;
        }
    }
    results(checker, &receivers).await
}

async fn computed_declaration_type(
    checker: &mut Checker<'_>,
    declaration: &NodeHandle,
) -> Option<TypeResponse> {
    let path = declaration.declaring_path()?;
    let position = {
        let encoded = checker.source_file(path.as_str()).await?;
        computed_key_position(encoded.as_bytes(), declaration)?
    };
    checker
        .session()
        .get_type_at_position(path.as_str(), position)
        .await
        .ok()
        .flatten()
}

fn computed_key_position(data: &[u8], declaration: &NodeHandle) -> Option<u32> {
    let (source, pos, end) = match declaration.parse() {
        Ok(parsed) => {
            let (source, _, _) = checker_node_text(data, 1).ok()?;
            (source, parsed.pos, parsed.end)
        }
        Err(_) => {
            let index = declaration
                .as_str()
                .split('.')
                .next()?
                .parse::<usize>()
                .ok()?;
            checker_node_text(data, index).ok()?
        }
    };
    let start = utf16_to_byte(source, pos)?;
    let stop = utf16_to_byte(source, end)?;
    let text = source.get(start..stop)?;
    let open = text.find('[')?;
    let close = text.get(open + 1..)?.find(']')?.saturating_add(open + 1);
    let colon = text.find(':').unwrap_or(text.len());
    if close > colon {
        return None;
    }
    let inside = text.get(open + 1..close)?;
    let leading = inside.len().saturating_sub(inside.trim_start().len());
    let mapped = inside.find(" in ").map(|offset| offset + 4);
    let relative = open + 1 + mapped.unwrap_or(leading);
    let prefix = text.get(..relative)?;
    Some(pos.saturating_add(prefix.encode_utf16().count() as u32))
}

fn utf16_to_byte(text: &str, offset: u32) -> Option<usize> {
    let mut units = 0u32;
    for (index, character) in text.char_indices() {
        if units >= offset {
            return Some(index);
        }
        units += character.len_utf16() as u32;
    }
    (units >= offset).then_some(text.len())
}

async fn mismatch(
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
        if depth > MAX_DEPTH {
            return false;
        }
        let sender = checker.base_constraint(&sender).await.unwrap_or(sender);
        let receiver = checker.base_constraint(&receiver).await.unwrap_or(receiver);
        if sender.id == receiver.id {
            return false;
        }
        if !visited.insert((sender.id.clone(), receiver.id.clone())) {
            return false;
        }
        if direct_mismatch(checker, &sender, &receiver).await {
            return true;
        }
        let sender_arguments = reference_arguments(checker, &sender).await;
        let receiver_arguments = reference_arguments(checker, &receiver).await;
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
        if let (Some(sender), Some(receiver)) = (
            number_index(checker, &sender).await,
            number_index(checker, &receiver).await,
        ) && deep_mismatch(
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
        let sender = checker.apparent(&sender).await;
        let receiver = checker.apparent(&receiver).await;
        let sender_properties = checker.properties(&sender).await;
        for receiver_property in checker.properties(&receiver).await {
            let Some(sender_property) = sender_properties
                .iter()
                .find(|property| property.name == receiver_property.name)
            else {
                continue;
            };
            let (Some(sender), Some(receiver)) = (
                checker.type_of_symbol(sender_property).await,
                checker.type_of_symbol(&receiver_property).await,
            ) else {
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

async fn reference_arguments(checker: &mut Checker<'_>, ty: &TypeResponse) -> Vec<TypeResponse> {
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

async fn number_index(checker: &mut Checker<'_>, ty: &TypeResponse) -> Option<TypeResponse> {
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

async fn enum_names(checker: &mut Checker<'_>, ty: &TypeResponse) -> Vec<CompactString> {
    if let Some(known) = checker.enum_names.get(&ty.id) {
        return known.clone();
    }
    let mut visited = FxHashSet::default();
    let mut names = Vec::new();
    collect_enum_names(checker, ty.clone(), &mut visited, &mut names, 0).await;
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
    depth: u8,
) -> Boxed<'a, ()> {
    Box::pin(async move {
        if depth > MAX_DEPTH {
            return;
        }
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
            collect_enum_names(checker, argument, visited, names, depth + 1).await;
        }
        if let Some(element) = number_index(checker, &ty).await {
            collect_enum_names(checker, element, visited, names, depth + 1).await;
        }
        for property in checker.properties(&ty).await {
            if let Some(property) = checker.type_of_symbol(&property).await {
                collect_enum_names(checker, property, visited, names, depth + 1).await;
            }
        }
    })
}
