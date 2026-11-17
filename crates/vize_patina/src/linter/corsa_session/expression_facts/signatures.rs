//! Signature facts: the parameter types a call resolves to, and the return
//! types a function's call signatures declare.
use super::super::checker_memo::CheckerMemo;
use super::details::{DetailCache, type_detail};
use corsa::api::{NodeHandle, SignatureResponse, TypeResponse};
use serde_json::{Value, json};

// TypeScript 7 SignatureFlags / SignatureKind.
const HAS_REST_PARAMETER: u32 = 1;
const CALL_SIGNATURE: i32 = 0;
const CONSTRUCT_SIGNATURE: i32 = 1;
/// TypeScript 7 `TypeFlags.Boolean`.
const BOOLEAN: u32 = 1 << 8;
/// TypeScript 7 `TypeFlags.Union`.
const UNION: u32 = 1 << 27;
/// Calls with more argument positions than this keep only the first ones.
const MAX_POSITIONS: u32 = 64;

/// `{ parameterCount, hasRest, positions: [{ texts, detail } | null], rest? }`
/// for the signature `getResolvedSignature` picks at a call-like node.
/// `positions[i]` is the type of argument position `i`, rest parameters
/// expanded. With `with_rest`, `rest` is the type every argument after a
/// variadic tuple spread meets (`null` without a rest parameter).
pub(super) async fn resolved_parameters(
    session: &CheckerMemo<'_>,
    location: NodeHandle,
    (positions, with_rest): (u32, bool),
    cache: &mut DetailCache,
) -> corsa::Result<Option<Value>> {
    let signature = session
        .client()
        .get_resolved_signature(
            session.snapshot().handle.clone(),
            session.project_handle(),
            location,
        )
        .await
        .ok()
        .flatten();
    let Some(signature) = signature else {
        return Ok(None);
    };
    let count = u32::try_from(signature.parameters.len()).unwrap_or(u32::MAX);
    let has_rest = signature.flags & HAS_REST_PARAMETER != 0;
    let mut types = Vec::new();
    for index in 0..positions.min(MAX_POSITIONS) {
        if index >= count && !has_rest {
            break;
        }
        let parameter = session
            .client()
            .get_parameter_type(
                session.snapshot().handle.clone(),
                session.project_handle(),
                signature.id.clone(),
                i32::try_from(index).unwrap_or(i32::MAX),
            )
            .await
            .ok()
            .flatten();
        types.push(match parameter {
            Some(parameter) => described(session, &parameter, cache).await?,
            None => Value::Null,
        });
    }
    let mut parameters = json!({
        "parameterCount": count,
        "hasRest": has_rest,
        "positions": types,
    });
    if with_rest && let Some(object) = parameters.as_object_mut() {
        let rest = match rest_type(session, &signature, has_rest).await {
            Some(rest) => described(session, &rest, cache).await?,
            None => Value::Null,
        };
        object.insert("rest".into(), rest);
    }
    Ok(Some(parameters))
}

/// `FunctionSignature`'s rest type once the remaining arguments are consumed:
/// the last element of a tuple rest parameter, else the rest parameter's
/// element type (both on its base constraint).
async fn rest_type(
    session: &CheckerMemo<'_>,
    signature: &SignatureResponse,
    has_rest: bool,
) -> Option<TypeResponse> {
    if !has_rest {
        return None;
    }
    let client = session.client();
    let snapshot = session.snapshot().handle.clone();
    let project = session.project_handle();
    // The resolved signature's own parameter handles are registered only
    // once the runtime lists them (`getParametersOfSignature`).
    let parameter = client
        .get_parameters_of_signature(snapshot.clone(), project.clone(), signature.id.clone())
        .await
        .ok()?
        .pop()?;
    let declared = client
        .get_type_of_symbol(snapshot.clone(), project.clone(), parameter.id)
        .await
        .ok()??;
    let constrained = session.base_constraint(&declared).await.unwrap_or(declared);
    // Tuple references carry their element flags; `isTupleType` is not
    // served by every runtime.
    let tuple = !constrained.element_flags.is_empty()
        || constrained.fixed_length.is_some()
        || client
            .is_tuple_type(snapshot.clone(), project.clone(), constrained.id.clone())
            .await
            .unwrap_or(false);
    if tuple {
        return client
            .get_type_arguments(snapshot, project, constrained.id)
            .await
            .ok()?
            .pop();
    }
    let index = i32::try_from(signature.parameters.len().saturating_sub(1)).ok()?;
    client
        .get_parameter_type(snapshot, project, signature.id.clone(), index)
        .await
        .ok()
        .flatten()
}

/// `{ contextual, signatures: [{ texts, detail, predicate? }], ownSignatures? }`:
/// the return type of every call signature of the function's contextual type
/// (function expressions and arrows, when one exists) or of its own type.
/// With a contextual type, `ownSignatures` holds the function's own ones too.
/// With `predicates`, a signature with a type predicate (written or inferred)
/// describes the predicate's type as `predicate`.
pub(super) async fn function_returns(
    session: &CheckerMemo<'_>,
    own: &TypeResponse,
    location: Option<NodeHandle>,
    predicates: bool,
    cache: &mut DetailCache,
) -> corsa::Result<Value> {
    let contextual = match location {
        Some(location) => session.contextual_type(&location).await,
        None => None,
    };
    let function_type = contextual.as_ref().unwrap_or(own);
    let returns = signature_returns(session, function_type, predicates, cache).await?;
    Ok(match contextual {
        Some(_) => json!({
            "contextual": true,
            "signatures": returns,
            "ownSignatures": signature_returns(session, own, predicates, cache).await?,
        }),
        None => json!({ "contextual": false, "signatures": returns }),
    })
}

async fn signature_returns(
    session: &CheckerMemo<'_>,
    function_type: &TypeResponse,
    predicates: bool,
    cache: &mut DetailCache,
) -> corsa::Result<Vec<Value>> {
    // tsutils `getCallSignaturesOfType`: a union contributes the signatures of
    // each constituent (`(() => any) | null`).
    let constituents = if function_type.flags & UNION != 0 {
        session.constituents(function_type).await
    } else {
        vec![function_type.clone()]
    };
    let mut signatures = Vec::new();
    for constituent in &constituents {
        signatures.extend(
            session
                .get_signatures_of_type(constituent.id.clone(), CALL_SIGNATURE)
                .await
                .unwrap_or_default(),
        );
    }
    let mut returns = Vec::with_capacity(signatures.len());
    for signature in signatures {
        let return_type = session
            .client()
            .get_return_type_of_signature(
                session.snapshot().handle.clone(),
                session.project_handle(),
                signature.id.clone(),
            )
            .await
            .ok()
            .flatten();
        let Some(return_type) = return_type else {
            continue;
        };
        let mut described_return = described(session, &return_type, cache).await?;
        // Only a `boolean` return can carry a type predicate.
        if predicates
            && return_type.flags & BOOLEAN != 0
            && let Some(predicate) = predicate_type(session, &signature).await
            && let Value::Object(fields) = &mut described_return
        {
            fields.insert(
                "predicate".into(),
                described(session, &predicate, cache).await?,
            );
        }
        returns.push(described_return);
    }
    Ok(returns)
}

/// `checker.getTypePredicateOfSignature(signature)?.type`.
async fn predicate_type(
    session: &CheckerMemo<'_>,
    signature: &SignatureResponse,
) -> Option<TypeResponse> {
    session
        .client()
        .get_type_predicate_of_signature(
            session.snapshot().handle.clone(),
            session.project_handle(),
            signature.id.clone(),
        )
        .await
        .ok()
        .flatten()
        .and_then(|predicate| predicate.r#type)
}

async fn described(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut DetailCache,
) -> corsa::Result<Value> {
    let texts = session.render_type_texts(ty).await.unwrap_or_default();
    let texts = texts.iter().map(|text| text.as_str()).collect::<Vec<_>>();
    Ok(json!({ "texts": texts, "detail": type_detail(session, ty, cache).await? }))
}

/// `(call, construct)` signature counts of a callee typed as (a subtype of) the
/// global `Function` (named so, or carrying its `apply`/`call`/`bind`/`prototype`
/// members). typescript-eslint only calls such a value safe when it
/// also has signatures, which a local alias named `Function` does.
pub(super) async fn function_signature_counts<T: AsRef<str>>(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    texts: &[T],
    properties: &[T],
) -> Option<(usize, usize)> {
    let has = |name: &str| properties.iter().any(|property| property.as_ref() == name);
    let function_members = has("apply") && has("call") && has("bind") && has("prototype");
    if !function_members && !texts.iter().any(|text| text.as_ref().contains("Function")) {
        return None;
    }
    let call = session
        .get_signatures_of_type(ty.id.clone(), CALL_SIGNATURE)
        .await
        .ok()?;
    let construct = session
        .get_signatures_of_type(ty.id.clone(), CONSTRUCT_SIGNATURE)
        .await
        .ok()?;
    Some((call.len(), construct.len()))
}

/// `{ texts, detail }` of the checker's contextual type at `location`, or
/// `null` when there is none.
pub(super) async fn contextual_type(
    session: &CheckerMemo<'_>,
    location: NodeHandle,
    cache: &mut DetailCache,
) -> corsa::Result<Value> {
    let contextual = session.contextual_type(&location).await;
    match contextual {
        Some(contextual) => described(session, &contextual, cache).await,
        None => Ok(Value::Null),
    }
}
