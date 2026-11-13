//! Constrained type shapes: checker flags, literal values and union parts.
//!
//! typescript-eslint rules such as `no-unnecessary-condition` decide on
//! `getConstrainedTypeAtLocation` plus `TypeFlags`, not on rendered type text.
//! A shape carries those facts so the corsa rules can apply the same tests.
mod keys;
mod predicate;
use super::checker_memo::CheckerMemo;
use corsa::api::{IndexInfo, SymbolResponse, TypeHandle, TypePredicateResponse, TypeResponse};
use keys::{keyed_facts, keys};
pub(super) use predicate::predicate;
use serde_json::{Map, Value, json};

#[expect(
    clippy::disallowed_types,
    reason = "serde_json objects are keyed by std strings"
)]
type JsonMap = Map<std::string::String, Value>;
use vize_l0::FxHashMap;

// TypeScript 7 TypeFlags, from microsoft/typescript-go internal/checker/types.go.
const ANY: u32 = 1;
const UNKNOWN: u32 = 1 << 1;
const UNDEFINED: u32 = 1 << 2;
const NULL: u32 = 1 << 3;
const VOID: u32 = 1 << 4;
const STRING: u32 = 1 << 5;
const STRING_LIKE: u32 = STRING | STRING_LITERAL | TEMPLATE_LITERAL | STRING_MAPPING;
const INDEX: u32 = 1 << 21;
const TYPE_PARAMETER: u32 = 1 << 19;
const TEMPLATE_LITERAL: u32 = 1 << 22;
const STRING_MAPPING: u32 = 1 << 23;
const SUBSTITUTION: u32 = 1 << 24;
const INDEXED_ACCESS: u32 = 1 << 25;
const CONDITIONAL: u32 = 1 << 26;
const UNION: u32 = 1 << 27;
const INTERSECTION: u32 = 1 << 28;
const STRING_LITERAL: u32 = 1 << 10;
const NUMBER_LITERAL: u32 = 1 << 11;
const NULLABLE: u32 = ANY | UNKNOWN | UNDEFINED | NULL | VOID;
const CONSTRAINABLE: u32 = TYPE_PARAMETER
    | INDEXED_ACCESS
    | CONDITIONAL
    | SUBSTITUTION
    | UNION
    | INTERSECTION
    | TEMPLATE_LITERAL
    | STRING_MAPPING
    | INDEX;
// SymbolFlags.Optional.
const OPTIONAL: u32 = 1 << 24;
// SignatureKind.Call.
const CALL_SIGNATURE: i32 = 0;

/// What a rule needs beyond the constrained type's own flags.
#[derive(Clone, Default)]
pub(in crate::linter) struct ShapeRequest {
    /// Property name read off this expression (`expr.name`).
    pub member: Option<vize_l0::String>,
    /// The expression is a callee; record whether each union part's call
    /// signatures can return a nullable value.
    pub call: bool,
    /// Record whether each union part is an array or a tuple.
    pub array: bool,
    /// Describe the return types of the call signatures (a callback).
    pub returns: bool,
    /// Generated range of the key expression when this is the object of a
    /// computed member access (`obj[key]`): record per-key property facts.
    pub keys_from: Option<(u32, u32)>,
    /// Generated ranges of the callee and of the arguments before the first
    /// spread when this is a call: record its type predicate.
    pub predicate: Option<PredicateRequest>,
}

impl ShapeRequest {
    /// Fold another request for the same expression into this one.
    pub(in crate::linter) fn merge(&mut self, other: Self) {
        if self.member.is_none() {
            self.member = other.member;
        }
        self.call |= other.call;
        self.array |= other.array;
        self.returns |= other.returns;
        if self.keys_from.is_none() {
            self.keys_from = other.keys_from;
        }
        if self.predicate.is_none() {
            self.predicate = other.predicate;
        }
    }
}

/// Operands of a call whose type predicate (`x is T` / `asserts x`) a rule
/// checks.
#[derive(Clone, Default)]
pub(in crate::linter) struct PredicateRequest {
    pub callee: (u32, u32),
    pub arguments: Vec<(u32, u32)>,
}

/// Per-batch memo of checker answers keyed by type handle.
#[derive(Default)]
pub(super) struct ShapeCache {
    constraints: FxHashMap<TypeHandle, Option<TypeResponse>>,
    constituents: FxHashMap<TypeHandle, Vec<TypeResponse>>,
    predicates: FxHashMap<TypeHandle, Option<TypePredicateResponse>>,
}

/// The position facts of `ty` for the `__typeContext` fact: property facts
/// of the constrained type (and of each union constituent) for `request`,
/// computed keys, and the call-signature return types to describe as trees.
/// The type itself goes into the node's `__type` tree.
pub(super) async fn describe(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    request: &ShapeRequest,
    key: Option<&TypeResponse>,
    cache: &mut ShapeCache,
) -> corsa::Result<(Value, Vec<TypeResponse>)> {
    let constrained = match constraint(session, ty, cache).await? {
        Some(constrained) => constrained,
        None => ty.clone(),
    };
    let mut context = Map::new();
    let is_union = constrained.flags & UNION != 0;
    let keys = match key {
        Some(key) => Some(keys(session, key, cache).await?),
        None => None,
    };
    // Per-part facts only matter for unions: they tell an optional chain
    // whether the nullability comes from the object itself.
    let part_facts = is_union && (request.member.is_some() || request.call || keys.is_some());
    if part_facts {
        let parts = constituents(session, &constrained, cache).await?;
        let mut entries = Vec::with_capacity(parts.len());
        for part in &parts {
            let mut entry = Map::new();
            if let Some(name) = request.member.as_deref() {
                entry.insert("member".into(), property(session, part, name, cache).await?);
            }
            if request.call {
                entry.insert(
                    "callReturnsNullable".into(),
                    json!(call_returns_nullable(session, part, cache).await?),
                );
            }
            if let Some(keys) = &keys {
                keyed_facts(session, part, keys, &mut entry, cache).await?;
            }
            entries.push(Value::Object(entry));
        }
        context.insert("parts".into(), Value::Array(entries));
    }
    if let Some(keys) = keys {
        keyed_facts(session, &constrained, &keys, &mut context, cache).await?;
        context.insert(
            "keys".into(),
            Value::Array(keys.into_iter().map(|key| key.json).collect()),
        );
    }
    if let Some(name) = request.member.as_deref() {
        let member = property(session, &constrained, name, cache).await?;
        context.insert("member".into(), member);
    }
    let returns = if request.returns {
        return_types(session, &constrained).await
    } else {
        Vec::new()
    };
    Ok((Value::Object(context), returns))
}

/// The call-signature return types, which typescript-eslint's
/// `getConstraintInfo` reads through their constraints.
async fn return_types(session: &CheckerMemo<'_>, ty: &TypeResponse) -> Vec<TypeResponse> {
    let signatures = session
        .get_signatures_of_type(ty.id.clone(), CALL_SIGNATURE)
        .await
        .unwrap_or_default();
    let mut types = Vec::with_capacity(signatures.len());
    for signature in signatures {
        if let Some(returns) = session
            .get_return_type_of_signature(signature.id.clone())
            .await
            .unwrap_or(None)
        {
            types.push(returns);
        }
    }
    types
}

async fn constraint(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut ShapeCache,
) -> corsa::Result<Option<TypeResponse>> {
    if ty.flags & CONSTRAINABLE == 0 {
        return Ok(None);
    }
    if let Some(found) = cache.constraints.get(&ty.id) {
        return Ok(found.clone());
    }
    let found = session.base_constraint(ty).await;
    cache.constraints.insert(ty.id.clone(), found.clone());
    Ok(found)
}

async fn constituents(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut ShapeCache,
) -> corsa::Result<Vec<TypeResponse>> {
    if let Some(found) = cache.constituents.get(&ty.id) {
        return Ok(found.clone());
    }
    let found = session.constituents(ty).await;
    cache.constituents.insert(ty.id.clone(), found.clone());
    Ok(found)
}

/// `{ found, optional, nullable, stringIndexNullable }` for `name` on `ty`.
async fn property(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    name: &str,
    cache: &mut ShapeCache,
) -> corsa::Result<Value> {
    let symbol: Option<SymbolResponse> = session
        .client()
        .get_property_of_type(
            session.snapshot().handle.clone(),
            session.project_handle(),
            ty.id.clone(),
            name,
        )
        .await
        .unwrap_or(None);
    let mut entry = Map::new();
    entry.insert("found".into(), json!(symbol.is_some()));
    if let Some(symbol) = symbol {
        entry.insert("optional".into(), json!(symbol.flags & OPTIONAL != 0));
        let property_type = session
            .get_type_of_symbol(symbol.id.clone())
            .await
            .unwrap_or(None);
        let (nullable, non_nullish) = match property_type {
            Some(property_type) => (
                is_nullable(session, &property_type, cache).await?,
                is_possibly_non_nullish(session, &property_type, cache).await?,
            ),
            None => (false, true),
        };
        entry.insert("nullable".into(), json!(nullable));
        entry.insert("nonNullish".into(), json!(non_nullish));
    } else {
        let infos: Vec<IndexInfo> = session
            .client()
            .get_index_infos_of_type(
                session.snapshot().handle.clone(),
                session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or_default();
        let mut string_index_nullable = None;
        for info in infos {
            // typescript-eslint `getTypeName(keyType) === 'string'`.
            if info.key_type.flags & STRING_LIKE == 0 {
                continue;
            }
            let nullable = is_nullable(session, &info.value_type, cache).await?;
            string_index_nullable = Some(string_index_nullable.unwrap_or(false) || nullable);
        }
        if let Some(nullable) = string_index_nullable {
            entry.insert("stringIndexNullable".into(), json!(nullable));
        }
    }
    Ok(Value::Object(entry))
}

async fn call_returns_nullable(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut ShapeCache,
) -> corsa::Result<bool> {
    let signatures = session
        .get_signatures_of_type(ty.id.clone(), CALL_SIGNATURE)
        .await
        .unwrap_or_default();
    for signature in signatures {
        let returns = session
            .get_return_type_of_signature(signature.id.clone())
            .await
            .unwrap_or(None);
        if let Some(returns) = returns
            && is_nullable(session, &returns, cache).await?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// typescript-eslint `isNullableType`: any, unknown, null, undefined or void
/// anywhere in the union.
async fn is_nullable(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut ShapeCache,
) -> corsa::Result<bool> {
    if ty.flags & NULLABLE != 0 {
        return Ok(true);
    }
    if ty.flags & UNION == 0 {
        return Ok(false);
    }
    Ok(constituents(session, ty, cache)
        .await?
        .iter()
        .any(|part| part.flags & NULLABLE != 0))
}

/// no-unnecessary-condition `isPossiblyNonNullish`: some union part is not
/// null, undefined or void.
async fn is_possibly_non_nullish(
    session: &CheckerMemo<'_>,
    ty: &TypeResponse,
    cache: &mut ShapeCache,
) -> corsa::Result<bool> {
    const NULLISH: u32 = UNDEFINED | NULL | VOID;
    if ty.flags & UNION == 0 {
        return Ok(ty.flags & NULLISH == 0);
    }
    Ok(constituents(session, ty, cache)
        .await?
        .iter()
        .any(|part| part.flags & NULLISH == 0))
}
