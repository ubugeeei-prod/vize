//! The contextual half of typescript-eslint's `no-unnecessary-type-assertion`:
//! `shouldSkipContextualTypeFallback` and `isContextuallyUnnecessary`.
use super::checker::{CALL, Checker, flags};
use super::unchanged::{Found, contains, is_empty_object_type};
use super::{AssertionAsk, Located, Span};
use corsa::api::TypeResponse;

/// `checker.getContextualType(node)` unless the fallback is skipped.
pub(super) async fn fallback_type(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &AssertionAsk,
    cast: &TypeResponse,
    uncast: &TypeResponse,
    translate: &dyn Fn(Span) -> Span,
) -> Option<TypeResponse> {
    let context = &ask.context;
    let cast_is_any = cast.flags & flags::ANY != 0 && !context.skip_parent;
    let skip = if cast_is_any {
        context.parent_logical || in_generic_context(checker, located, ask, translate).await
    } else {
        context.skip
            || property_in_problematic_context(checker, located, ask, uncast, translate).await
            || argument_to_overloaded_function(checker, located, ask, uncast, translate).await
            || (!context.literal_original
                && !context.parent_property
                && in_generic_context(checker, located, ask, translate).await)
    };
    if skip {
        return None;
    }
    let handle = located.handle(translate(ask.node))?;
    checker.contextual(handle).await
}

pub(super) async fn is_contextually_unnecessary(
    checker: &mut Checker<'_>,
    ask: &AssertionAsk,
    cast: &TypeResponse,
    uncast: &TypeResponse,
    contextual: &TypeResponse,
    located: &Located,
    translate: &dyn Fn(Span) -> Span,
) -> bool {
    let context = &ask.context;
    if context.const_assertion {
        return false;
    }
    let cast_is_any = cast.flags & flags::ANY != 0 && !context.skip_parent;
    let contextual_is_any = contextual.flags & flags::ANY != 0;
    if context.nullish_literal && cast.flags & flags::UNION != 0 {
        return false;
    }
    // The cheap relation first: most assertions stop here.
    if !contextual_is_any && !checker.assignable(uncast, contextual).await {
        return false;
    }
    if contains(checker, uncast, flags::ANY).await != Found::No {
        return false;
    }
    let any_involved = if contextual_is_any {
        context.call_argument && contains(checker, cast, flags::ANY).await == Found::No
    } else {
        contains(checker, contextual, flags::ANY).await == Found::No
    };
    if !any_involved {
        return false;
    }
    if phantom_type_argument_mismatch(checker, located, ask, uncast, contextual, translate).await {
        return false;
    }
    cast_is_any || !generics_mismatch(checker, uncast, contextual).await
}

async fn in_generic_context(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &AssertionAsk,
    translate: &dyn Fn(Span) -> Span,
) -> bool {
    for callee in &ask.context.generic_callees {
        let Some(ty) = located.ty(translate(*callee)).cloned() else {
            continue;
        };
        if has_generic_call_signature(checker, &ty).await {
            return true;
        }
    }
    false
}

async fn has_generic_call_signature(checker: &mut Checker<'_>, ty: &TypeResponse) -> bool {
    checker
        .signatures(ty, CALL)
        .await
        .iter()
        .any(|signature| !signature.type_parameters.is_empty())
}

/// `isPropertyInProblematicContext`.
async fn property_in_problematic_context(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &AssertionAsk,
    uncast: &TypeResponse,
    translate: &dyn Fn(Span) -> Span,
) -> bool {
    let Some(property) = &ask.context.property else {
        return false;
    };
    let object = match located.handle(translate(property.object)) {
        Some(handle) => checker.contextual(handle).await,
        None => return true,
    };
    if !object.is_some_and(|object| object.flags & flags::UNION != 0) {
        return property.satisfies;
    }
    let Some(handle) = located.handle(translate(ask.node)) else {
        return true;
    };
    let Some(own) = checker.contextual(handle).await else {
        return true;
    };
    let Some(non_nullable) = checker.non_nullable(&own).await else {
        return true;
    };
    if non_nullable.flags & flags::UNION != 0 {
        return true;
    }
    !checker.assignable(uncast, &non_nullable).await
}

/// `isArgumentToOverloadedFunction`.
async fn argument_to_overloaded_function(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &AssertionAsk,
    uncast: &TypeResponse,
    translate: &dyn Fn(Span) -> Span,
) -> bool {
    let Some((callee, index)) = ask.context.overload else {
        return false;
    };
    let Some(callee) = located.ty(translate(callee)).cloned() else {
        return true;
    };
    let Some(callee) = checker.non_nullable(&callee).await else {
        return true;
    };
    let signatures = checker.signatures(&callee, CALL).await;
    if signatures.len() <= 1 {
        return false;
    }
    let mut parameter_types = Vec::new();
    for signature in &signatures {
        let parameters = checker.parameter_types(signature).await;
        if parameters.len() != signature.parameters.len() {
            return true;
        }
        let Some(parameter) = parameters.get(index).cloned() else {
            return true;
        };
        // A rest parameter contributes its element type.
        let rest = signature.flags & 1 != 0 && index + 1 == parameters.len();
        let parameter = if rest {
            checker
                .type_arguments(&parameter)
                .await
                .into_iter()
                .next()
                .unwrap_or(parameter)
        } else {
            parameter
        };
        parameter_types.push(parameter);
    }
    let Some(first) = parameter_types.first() else {
        return false;
    };
    if parameter_types
        .iter()
        .all(|parameter| parameter.id == first.id)
    {
        return false;
    }
    for parameter in &parameter_types {
        if !checker.assignable(uncast, parameter).await {
            return true;
        }
    }
    false
}

/// `hasPhantomTypeArgumentMismatch`.
async fn phantom_type_argument_mismatch(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &AssertionAsk,
    uncast: &TypeResponse,
    contextual: &TypeResponse,
    translate: &dyn Fn(Span) -> Span,
) -> bool {
    if ask.context.generic_callees.is_empty()
        || !in_generic_context(checker, located, ask, translate).await
    {
        return false;
    }
    let uncast_arguments = checker.type_arguments(uncast).await;
    let contextual_arguments = checker.type_arguments(contextual).await;
    let phantom = (!uncast_arguments.is_empty() && is_empty_object_type(checker, uncast).await)
        || (!contextual_arguments.is_empty() && is_empty_object_type(checker, contextual).await);
    phantom
        && !(uncast_arguments.len() == contextual_arguments.len()
            && uncast_arguments
                .iter()
                .zip(&contextual_arguments)
                .all(|(left, right)| left.id == right.id))
}

/// `genericsMismatch`: a generic method of the contextual type that the
/// operand only has in a non-generic form.
async fn generics_mismatch(
    checker: &mut Checker<'_>,
    uncast: &TypeResponse,
    contextual: &TypeResponse,
) -> bool {
    for property in checker.properties(contextual).await {
        let Some(ty) = checker.type_of_symbol(&property).await else {
            continue;
        };
        if !has_generic_call_signature(checker, &ty).await {
            continue;
        }
        let Some(own) = checker.property(uncast, &property.name).await else {
            return true;
        };
        let Some(own) = checker.type_of_symbol(&own).await else {
            return true;
        };
        if !has_generic_call_signature(checker, &own).await {
            return true;
        }
    }
    false
}
