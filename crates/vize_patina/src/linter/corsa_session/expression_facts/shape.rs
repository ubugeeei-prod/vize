//! The `__type` fact: one recursive checker type tree per type.
//!
//! typescript-eslint rules inspect `TypeFlags`, union and intersection
//! constituents, base constraints, literal values, type references and their
//! arguments, enum identity, base types and where `toString` comes from.
//! Rendered type text cannot answer those questions, so the host sends a small
//! JSON tree (schema in corsa's `lint::type_tree`). Each type is built once per
//! batch, with the union of the details every fact family asked of it.
mod members;
mod object;
use super::super::checker_memo::CheckerMemo;
use corsa::api::{NodeHandle, TypeHandle, TypeResponse};
use serde_json::{Map, Value, json};
use std::{future::Future, pin::Pin};
use vize_l0::FxHashMap;

// TypeScript 7 TypeFlags, from microsoft/typescript-go internal/checker/types.go.
const VOID: u32 = 1 << 4;
const NUMBER: u32 = 1 << 6;
const BIGINT_LITERAL: u32 = 1 << 12;
const STRING_LITERAL: u32 = 1 << 10;
const NUMBER_LITERAL: u32 = 1 << 11;
const BOOLEAN_LITERAL: u32 = 1 << 13;
const ENUM_LITERAL: u32 = 1 << 15;
const ENUM: u32 = 1 << 16;
const NON_PRIMITIVE: u32 = 1 << 17;
const TYPE_PARAMETER: u32 = 1 << 19;
const OBJECT: u32 = 1 << 20;
const INDEX: u32 = 1 << 21;
const TEMPLATE_LITERAL: u32 = 1 << 22;
const STRING_MAPPING: u32 = 1 << 23;
const SUBSTITUTION: u32 = 1 << 24;
const INDEXED_ACCESS: u32 = 1 << 25;
const CONDITIONAL: u32 = 1 << 26;
const UNION: u32 = 1 << 27;
const INTERSECTION: u32 = 1 << 28;
const NUMBER_LIKE: u32 = NUMBER | NUMBER_LITERAL | ENUM;
/// Types `getBaseConstraintOfType` can answer with another type.
const CONSTRAINABLE: u32 = TYPE_PARAMETER
    | INDEX
    | SUBSTITUTION
    | INDEXED_ACCESS
    | CONDITIONAL
    | UNION
    | INTERSECTION
    | TEMPLATE_LITERAL
    | STRING_MAPPING;

// ObjectFlags.
const CLASS: u32 = 1 << 0;
const INTERFACE: u32 = 1 << 1;
const REFERENCE: u32 = 1 << 2;
const TUPLE: u32 = 1 << 3;

// SymbolFlags.Class and SymbolFlags.Interface.
const SYMBOL_CLASS: u32 = 1 << 5;
const SYMBOL_INTERFACE: u32 = 1 << 6;

/// Nesting depth of the tree: deep enough for a type argument's union
/// constituent's intersection members, and for `T[]` inside a union inside a
/// constraint.
const MAX_DEPTH: u8 = 6;
/// Type references below this depth list their type arguments.
const MAX_ARGUMENT_DEPTH: u8 = 4;
const BASE_DEPTH: u8 = 6;
const COERCION_METHODS: [&str; 3] = ["toLocaleString", "toString", "valueOf"];

#[expect(
    clippy::disallowed_types,
    reason = "serde_json objects are keyed by std strings"
)]
type JsonMap = Map<std::string::String, Value>;

type Boxed<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Which optional parts of a type tree to load. Flags, constituents, base
/// constraints and literal values are always included; each detail costs
/// extra checker requests, so a query asks only for what its rules read.
pub(in crate::linter) mod detail {
    /// Load a tree at all.
    pub(in crate::linter) const STRUCTURE: u16 = 1 << 15;
    /// The type's symbol: name, flags and default-library origin.
    pub(in crate::linter) const SYMBOL: u16 = 1 << 0;
    /// Array element and tuple element types (type arguments of an array or
    /// tuple reference).
    pub(in crate::linter) const ELEMENTS: u16 = 1 << 1;
    /// Names of the transitive base types.
    pub(in crate::linter) const BASES: u16 = 1 << 2;
    /// Whether the type has call signatures.
    pub(in crate::linter) const CALLABLE: u16 = 1 << 3;
    /// Construct signatures (own and of the symbol's type), property count,
    /// `[Symbol.iterator]`.
    pub(in crate::linter) const OBJECT_KIND: u16 = 1 << 4;
    /// Where `toString`, `toLocaleString` and `valueOf` come from.
    pub(in crate::linter) const TO_STRING: u16 = 1 << 5;
    /// The rendered text of every type in the tree.
    pub(in crate::linter) const TEXT: u16 = 1 << 6;
    /// Target, type arguments and text of every type reference.
    pub(in crate::linter) const REFERENCE: u16 = 1 << 7;
    /// `isArrayType` / `isTupleType` of object types.
    pub(in crate::linter) const ARRAY: u16 = 1 << 8;
    /// The enum type of an enum literal.
    pub(in crate::linter) const ENUM: u16 = 1 << 9;
    /// A `number` index signature and a number-like `length`.
    pub(in crate::linter) const NUMBER_INDEX: u16 = 1 << 10;

    /// What the type-structure rules (`no-base-to-string`, `restrict-*`,
    /// `no-implied-eval`, `no-misused-spread`, ...) read: every text.
    pub(in crate::linter) const TREE: u16 = STRUCTURE | TEXT;
    /// What the `no-unsafe-*` and enum rules read.
    pub(in crate::linter) const CHECKER: u16 = STRUCTURE | REFERENCE | ARRAY | ENUM;
    /// What the `no-unnecessary-*` rules read through the constraint.
    pub(in crate::linter) const CONSTRAINED: u16 = STRUCTURE;
    /// What `no-for-in-array`, `no-array-delete` and
    /// `prefer-reduce-type-parameter` read through the constraint.
    pub(in crate::linter) const ARRAY_LIKE: u16 = STRUCTURE | ARRAY | NUMBER_INDEX;
}

/// Builds and caches shape trees for one snapshot.
pub(super) struct Shapes<'a> {
    session: &'a CheckerMemo<'a>,
    file: &'a str,
    /// Trees by type, with the depth and details they were built with.
    cache: FxHashMap<TypeHandle, (u8, u16, Value)>,
    object_members: Option<FxHashMap<&'static str, Vec<NodeHandle>>>,
    /// unbound-method facts by receiver type and member name.
    member_cache: FxHashMap<(TypeHandle, vize_l0::String), Value>,
    /// `checkIfMethod` inputs by property symbol.
    method_cache: FxHashMap<corsa::api::SymbolHandle, Value>,
    /// Receiver symbol facts by type.
    receiver_cache: FxHashMap<TypeHandle, JsonMap>,
    /// Whether a reference target is a tuple type, by target.
    tuple_targets: FxHashMap<TypeHandle, bool>,
}

impl<'a> Shapes<'a> {
    pub(super) fn new(session: &'a CheckerMemo<'a>, file: &'a str) -> Self {
        Self {
            session,
            file,
            cache: FxHashMap::default(),
            object_members: None,
            member_cache: FxHashMap::default(),
            method_cache: FxHashMap::default(),
            receiver_cache: FxHashMap::default(),
            tuple_targets: FxHashMap::default(),
        }
    }

    /// The `__type` tree of `ty` with `details` (`detail::*`).
    pub(super) async fn shape(&mut self, ty: &TypeResponse, details: u16) -> Value {
        self.build(ty.clone(), 0, details | detail::STRUCTURE).await
    }

    fn build(&mut self, ty: TypeResponse, depth: u8, details: u16) -> Boxed<'_, Value> {
        Box::pin(async move {
            let mut details = details;
            if let Some((built_at, built_with, cached)) = self.cache.get(&ty.id) {
                if *built_at <= depth && built_with & details == details {
                    return cached.clone();
                }
                // Rebuild once with everything asked of this type so far.
                if *built_at <= depth {
                    details |= *built_with;
                }
            }
            let shape = self.compute(&ty, depth, details).await;
            self.cache
                .insert(ty.id.clone(), (depth, details, shape.clone()));
            shape
        })
    }

    async fn compute(&mut self, ty: &TypeResponse, depth: u8, details: u16) -> Value {
        let flags = ty.flags;
        let object_flags = ty.object_flags.unwrap_or(0);
        let reference = object_flags & REFERENCE != 0;
        let mut out = Map::new();
        out.insert("id".into(), json!(ty.id.as_str()));
        out.insert("flags".into(), json!(flags));
        if ty.object_flags.is_some() {
            out.insert("objectFlags".into(), json!(object_flags));
        }
        if details & detail::TEXT != 0 || (reference && details & detail::REFERENCE != 0) {
            out.insert("text".into(), json!(self.text(ty).await));
        }
        if let Some(value) = self.literal_value(ty).await {
            out.insert("value".into(), value);
        }
        if depth >= MAX_DEPTH {
            return Value::Object(out);
        }
        // A union or intersection of non-generic types is its own constraint.
        let mut constrainable = flags & CONSTRAINABLE & !(UNION | INTERSECTION) != 0;
        if flags & (UNION | INTERSECTION) != 0 {
            let parts = self.session.constituents(ty).await;
            constrainable |= parts.iter().any(|part| part.flags & CONSTRAINABLE != 0);
            let mut shapes = Vec::with_capacity(parts.len());
            for part in parts {
                shapes.push(self.build(part, depth + 1, details).await);
            }
            out.insert("types".into(), Value::Array(shapes));
        }
        if constrainable
            && let Some(constraint) = self
                .session
                .base_constraint(ty)
                .await
                .filter(|constraint| constraint.id != ty.id)
        {
            let shape = self.build(constraint, depth + 1, details).await;
            out.insert("constraint".into(), shape);
        }
        if flags & ENUM_LITERAL != 0 && details & detail::ENUM != 0 {
            self.enum_of(ty, &mut out).await;
        }
        if flags & OBJECT != 0 {
            self.references(ty, depth, details, &mut out).await;
            self.object_facts(ty, details, &mut out).await;
            if details & detail::NUMBER_INDEX != 0 {
                self.number_index(ty, &mut out).await;
            }
        }
        if flags & (OBJECT | NON_PRIMITIVE) != 0 && details & detail::TO_STRING != 0 {
            out.insert("toString".into(), self.object_coercion(ty).await);
        }
        Value::Object(out)
    }

    /// `isArrayType`, `isTupleType`, and a reference's target and type
    /// arguments (always for `REFERENCE`, for arrays and tuples for
    /// `ELEMENTS`).
    async fn references(&mut self, ty: &TypeResponse, depth: u8, details: u16, out: &mut JsonMap) {
        let wants_arguments = details & (detail::REFERENCE | detail::ELEMENTS) != 0;
        if details & detail::ARRAY == 0 && !wants_arguments {
            return;
        }
        let reference = ty.object_flags.unwrap_or(0) & REFERENCE != 0;
        if !reference {
            // `isArrayType` and `isTupleType` need a type reference.
            out.insert("array".into(), json!(false));
            out.insert("tuple".into(), json!(false));
            return;
        }
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let array = client
            .is_array_type(snapshot.clone(), project.clone(), ty.id.clone())
            .await
            .unwrap_or(false);
        // `checker.isTupleType`: a reference whose target is a tuple type.
        // The API's own predicate answers false for tuple references.
        let tuple = !array
            && (!ty.element_flags.is_empty()
                || ty.fixed_length.is_some()
                || self.is_tuple_target(ty).await);
        out.insert("array".into(), json!(array));
        out.insert("tuple".into(), json!(tuple));
        if let Some(target) = &ty.target {
            out.insert("target".into(), json!(target.as_str()));
        }
        let arguments = details & detail::REFERENCE != 0
            || (details & detail::ELEMENTS != 0 && (array || tuple));
        if !arguments || depth >= MAX_ARGUMENT_DEPTH {
            return;
        }
        let types = client
            .get_type_arguments(snapshot, project, ty.id.clone())
            .await
            .unwrap_or_default();
        let mut shapes = Vec::with_capacity(types.len());
        for argument in types {
            shapes.push(self.build(argument, depth + 1, details).await);
        }
        out.insert("typeArguments".into(), Value::Array(shapes));
    }

    /// Whether the target of reference `ty` is a tuple type, once per target.
    async fn is_tuple_target(&mut self, ty: &TypeResponse) -> bool {
        if let Some(known) = ty
            .target
            .as_ref()
            .and_then(|target| self.tuple_targets.get(target))
        {
            return *known;
        }
        let target = self
            .session
            .client()
            .get_target_of_type_in_project(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .ok()
            .flatten();
        let tuple = target
            .as_ref()
            .is_some_and(|target| target.object_flags.unwrap_or(0) & TUPLE != 0);
        if let Some(target) = ty.target.clone().or_else(|| target.map(|target| target.id)) {
            self.tuple_targets.insert(target, tuple);
        }
        tuple
    }

    /// The enum an enum literal belongs to (`getBaseTypeOfLiteralType`).
    async fn enum_of(&self, ty: &TypeResponse, out: &mut JsonMap) {
        let base = self
            .session
            .client()
            .get_base_type_of_literal_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .ok()
            .flatten();
        if let Some(base) = base {
            out.insert(
                "enum".into(),
                json!({ "id": base.id.as_str(), "text": self.text(&base).await }),
            );
        }
    }

    /// A literal's value: strings and numbers as JSON, bigints as
    /// `{ "bigint": text }`, boolean literals as JSON booleans.
    async fn literal_value(&self, ty: &TypeResponse) -> Option<Value> {
        if ty.flags & BOOLEAN_LITERAL != 0 && ty.flags & UNION == 0 {
            return Some(json!(self.text(ty).await.trim() == "true"));
        }
        if ty.flags & (STRING_LITERAL | NUMBER_LITERAL | BIGINT_LITERAL) == 0 {
            return None;
        }
        let value = ty.value.clone()?;
        if ty.flags & BIGINT_LITERAL != 0 {
            return Some(json!({ "bigint": bigint_text(&value) }));
        }
        Some(value)
    }

    async fn text(&self, ty: &TypeResponse) -> vize_l0::String {
        self.session
            .type_texts(ty)
            .await
            .into_iter()
            .next()
            .unwrap_or_default()
    }
}

/// A bigint literal's digits, `-` first when negative.
fn bigint_text(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let negative = map
                .get("negative")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let digits = map
                .get("base10Value")
                .and_then(Value::as_str)
                .unwrap_or("0");
            if negative && digits != "0" {
                Value::String(["-", digits].concat())
            } else {
                Value::String(digits.into())
            }
        }
        Value::String(text) => Value::String(text.trim_end_matches('n').into()),
        other => other.clone(),
    }
}
