//! Per-ask relation answers: type assertions, non-null assertions and return types.
use super::checker::Checker;
use super::{
    AssertionAsk, AssertionReply, ContextSource, Literal, Located, NonNullAsk, NonNullReply,
    ReturnAsk, ReturnReply, Span, contextual, flags, unchanged,
};
use corsa::api::TypeResponse;

pub(super) async fn assertion(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &AssertionAsk,
    translate: &dyn Fn(Span) -> Span,
) -> AssertionReply {
    let Some(cast) = located.ty(translate(ask.node)).cloned() else {
        return AssertionReply::default();
    };
    let mut reply = AssertionReply {
        cast_is_literal: cast.flags & flags::LITERAL != 0,
        ..AssertionReply::default()
    };
    let uncast = match ask.iife {
        Some((callee, annotated)) => match located.ty(translate(callee)) {
            Some(callee) => checker.iife_return(callee, annotated).await,
            None => None,
        },
        None => located.ty(translate(ask.expression)).cloned(),
    };
    let Some(uncast) = uncast else {
        return reply;
    };
    reply.unchanged =
        unchanged::is_type_unchanged(checker, ask.intersection, ask.literal, &uncast, &cast).await;
    let would_same_type_be_inferred = if reply.cast_is_literal {
        ask.context.narrowed_declaration
    } else {
        !ask.context.const_assertion
    };
    if reply.unchanged && would_same_type_be_inferred {
        return reply;
    }
    let contextual =
        contextual::fallback_type(checker, located, ask, &cast, &uncast, translate).await;
    if let Some(contextual) = &contextual {
        reply.contextually_unnecessary = contextual::is_contextually_unnecessary(
            checker, ask, &cast, &uncast, contextual, located, translate,
        )
        .await;
        if reply.contextually_unnecessary {
            return reply;
        }
    }
    let Some(original) = ask.original.and_then(|span| located.ty(translate(span))) else {
        return reply;
    };
    // `isTypeUnchanged(node, innerExpression, ...)`: the inner assertion is
    // never a literal.
    reply.double_unchanged = cast.flags & flags::ANY == 0
        && unchanged::is_type_unchanged(checker, ask.intersection, Literal::No, original, &cast)
            .await;
    if !reply.double_unchanged
        && let Some(contextual) = &contextual
    {
        // The intermediate type is the inner assertion's: the uncast type.
        reply.double_contextual = uncast.flags & (flags::ANY | flags::UNKNOWN) != 0
            && checker.assignable(original, contextual).await;
    }
    reply
}

pub(super) async fn non_null(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &NonNullAsk,
    translate: &dyn Fn(Span) -> Span,
) -> NonNullReply {
    let Some(actual) = located.ty(translate(ask.expression)).cloned() else {
        return NonNullReply::default();
    };
    let actual_flags = checker.union_flags(&actual).await;
    // Only instantiable types (and compositions of them) have a base
    // constraint other than themselves; skip the round-trip for the rest.
    let constrained = if (actual.flags | actual_flags) & flags::INSTANTIABLE != 0 {
        checker
            .base_constraint(&actual)
            .await
            .unwrap_or_else(|| actual.clone())
    } else {
        actual.clone()
    };
    let constrained_flags = if constrained.id == actual.id {
        actual_flags
    } else {
        checker.union_flags(&constrained).await
    };
    let mut reply = NonNullReply {
        typed: true,
        actual: actual_flags,
        constrained: constrained_flags,
        constrained_differs: constrained.id != actual.id,
        contextual: None,
        declared_matches: false,
    };
    if (actual_flags | constrained_flags) & flags::NULLABLE == 0 {
        if let Some(declared) = ask.declared.and_then(|span| located.ty(translate(span))) {
            reply.declared_matches = declared.id == constrained.id;
        }
        return reply;
    }
    if reply.constrained_differs {
        return reply;
    }
    let contextual = match ask.context {
        Some(ContextSource::Checker(span)) => match located.handle(translate(span)) {
            Some(handle) => checker.contextual(handle).await,
            None => None,
        },
        Some(ContextSource::TypeAt(span)) => located.ty(translate(span)).cloned(),
        None => None,
    };
    if let Some(contextual) = contextual {
        reply.contextual = Some(checker.union_flags(&contextual).await);
    }
    reply
}

pub(super) async fn returns(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &ReturnAsk,
    translate: &dyn Fn(Span) -> Span,
) -> ReturnReply {
    let mut reply = ReturnReply::default();
    let Some(class) = located.ty(translate(ask.class)).cloned() else {
        return reply;
    };
    // The class node may type as the constructor or the instance; both carry
    // the class symbol, as do its instance type and its `this` type.
    let Some(class_symbol) = class.symbol.clone() else {
        return reply;
    };
    let is_instance = |ty: &TypeResponse| {
        ty.id == class.id
            || (ty.flags & flags::OBJECT != 0 && ty.symbol.as_ref() == Some(&class_symbol))
    };
    for span in &ask.expressions {
        let Some(ty) = located.ty(translate(*span)) else {
            continue;
        };
        if ty.flags & flags::TYPE_PARAMETER != 0 && ty.symbol.as_ref() == Some(&class_symbol) {
            reply.this_type = true;
            continue;
        }
        if is_instance(ty) {
            reply.class_type = true;
            continue;
        }
        if ty.flags & flags::UNION != 0 && checker.parts(ty).await.iter().any(is_instance) {
            reply.class_type = true;
        }
    }
    reply
}
