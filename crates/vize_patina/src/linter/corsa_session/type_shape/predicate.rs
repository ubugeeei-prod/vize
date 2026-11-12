//! Type predicates of a call, for no-unnecessary-condition's `checkTypePredicates`.
use super::super::checker_memo::CheckerMemo;
use super::{
    CALL_SIGNATURE, CONSTRAINABLE, INTERSECTION, ShapeCache, ShapeRequest, UNION, constraint,
    describe,
};
use corsa::api::{TypePredicateResponse, TypeResponse};
use serde_json::{Map, Value, json};

// TypePredicateKind.Identifier and TypePredicateKind.AssertsIdentifier.
const PREDICATE_IDENTIFIER: i32 = 1;
const PREDICATE_ASSERTS_IDENTIFIER: i32 = 3;

/// The type predicate of a call, for no-unnecessary-condition's
/// `checkTypePredicates`. typescript-eslint reads the resolved signature;
/// this reads the callee's call signatures and answers only when they all
/// share one predicate, and never for a generic predicate type (whose
/// instantiation would need the resolved signature).
///
/// `{ "argument": index, "asserts": bool }` plus either `"shape"` (the
/// asserted argument's shape, for `asserts x`) or `"argumentFlags"`,
/// `"assignable"` (argument to predicate type), `"reverseAssignable"` and
/// `"predicateIsUnion"` (for `x is T` / `asserts x is T`).
pub(in super::super) async fn predicate(
    session: &CheckerMemo<'_>,
    callee: &TypeResponse,
    arguments: &[Option<&TypeResponse>],
    cache: &mut ShapeCache,
) -> corsa::Result<Option<Value>> {
    if arguments.is_empty() {
        return Ok(None);
    }
    let Some(found) = callee_predicate(session, callee, cache).await? else {
        return Ok(None);
    };
    let Ok(index) = usize::try_from(found.parameter_index) else {
        return Ok(None);
    };
    let Some(Some(argument)) = arguments.get(index) else {
        return Ok(None);
    };
    let asserts = found.kind == PREDICATE_ASSERTS_IDENTIFIER;
    let mut fact = Map::new();
    fact.insert("argument".into(), json!(index));
    fact.insert("asserts".into(), json!(asserts));
    match &found.r#type {
        None if asserts => {
            let request = ShapeRequest::default();
            let shape = Box::pin(describe(session, argument, &request, None, cache)).await?;
            fact.insert("shape".into(), shape);
        }
        Some(target) if found.kind == PREDICATE_IDENTIFIER || asserts => {
            if target.flags & CONSTRAINABLE & !(UNION | INTERSECTION) != 0 {
                return Ok(None);
            }
            let argument = match constraint(session, argument, cache).await? {
                Some(constrained) => constrained,
                None => (*argument).clone(),
            };
            fact.insert("argumentFlags".into(), json!(argument.flags));
            fact.insert(
                "assignable".into(),
                json!(assignable(session, &argument, target).await),
            );
            fact.insert(
                "reverseAssignable".into(),
                json!(assignable(session, target, &argument).await),
            );
            fact.insert("predicateIsUnion".into(), json!(target.flags & UNION != 0));
        }
        _ => return Ok(None),
    }
    Ok(Some(Value::Object(fact)))
}

pub(super) async fn assignable(
    session: &CheckerMemo<'_>,
    source: &TypeResponse,
    target: &TypeResponse,
) -> bool {
    session
        .client()
        .is_type_assignable_to(
            session.snapshot().handle.clone(),
            session.project_handle(),
            source.id.clone(),
            target.id.clone(),
        )
        .await
        .unwrap_or(false)
}

pub(super) async fn callee_predicate(
    session: &CheckerMemo<'_>,
    callee: &TypeResponse,
    cache: &mut ShapeCache,
) -> corsa::Result<Option<TypePredicateResponse>> {
    if let Some(found) = cache.predicates.get(&callee.id) {
        return Ok(found.clone());
    }
    let signatures = session
        .get_signatures_of_type(callee.id.clone(), CALL_SIGNATURE)
        .await
        .unwrap_or_default();
    let mut shared: Option<TypePredicateResponse> = None;
    let mut agree = !signatures.is_empty();
    for signature in &signatures {
        let found = session
            .client()
            .get_type_predicate_of_signature(
                session.snapshot().handle.clone(),
                session.project_handle(),
                signature.id.clone(),
            )
            .await
            .unwrap_or(None);
        let Some(found) = found else {
            agree = false;
            break;
        };
        if let Some(first) = &shared {
            if first.kind != found.kind
                || first.parameter_index != found.parameter_index
                || first.r#type.as_ref().map(|ty| &ty.id) != found.r#type.as_ref().map(|ty| &ty.id)
            {
                agree = false;
                break;
            }
        } else {
            shared = Some(found);
        }
    }
    let found = if agree { shared } else { None };
    cache.predicates.insert(callee.id.clone(), found.clone());
    Ok(found)
}
