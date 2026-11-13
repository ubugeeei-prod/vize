//! Structural checker facts for one expression type.
//!
//! typescript-eslint rules inspect `TypeFlags`, union and intersection
//! constituents, type-parameter constraints, array elements, base types and
//! where `toString` comes from. Rendered type text cannot answer those
//! questions, so the host sends them as a small JSON tree (`__typeShape`).
mod members;
use super::super::checker_memo::CheckerMemo;
use corsa::api::{DocumentIdentifier, NodeHandle, TypeHandle, TypeResponse};
use members::{declaration_kind, is_default_library};
use serde_json::{Map, Value, json};
use std::{future::Future, pin::Pin};
use vize_l0::FxHashMap;

// TypeScript 7 TypeFlags, from microsoft/typescript-go internal/checker/types.go.
const VOID: u32 = 1 << 4;
const NON_PRIMITIVE: u32 = 1 << 17;
const TYPE_PARAMETER: u32 = 1 << 19;
const OBJECT: u32 = 1 << 20;
const INDEX: u32 = 1 << 21;
const SUBSTITUTION: u32 = 1 << 24;
const INDEXED_ACCESS: u32 = 1 << 25;
const CONDITIONAL: u32 = 1 << 26;
const UNION: u32 = 1 << 27;
const INTERSECTION: u32 = 1 << 28;
const GENERIC: u32 = TYPE_PARAMETER | INDEX | SUBSTITUTION | INDEXED_ACCESS | CONDITIONAL;

// ObjectFlags.
const CLASS: u32 = 1 << 0;
const INTERFACE: u32 = 1 << 1;
const REFERENCE: u32 = 1 << 2;

// SymbolFlags.Class and SymbolFlags.Interface.
const SYMBOL_CLASS: u32 = 1 << 5;
const SYMBOL_INTERFACE: u32 = 1 << 6;
// TypeScript 7 SyntaxKind values.
const PROPERTY_DECLARATION: u16 = 174;
const METHOD_SIGNATURE: u16 = 175;
const METHOD_DECLARATION: u16 = 176;
const FUNCTION_EXPRESSION: u16 = 219;
const PROPERTY_ASSIGNMENT: u16 = 303;
const SHORTHAND_PROPERTY_ASSIGNMENT: u16 = 304;

/// Nesting depth of the shape tree. Deep enough for `T[]` inside a union
/// inside a constraint.
const MAX_DEPTH: u8 = 4;
const BASE_DEPTH: u8 = 6;
const COERCION_METHODS: [&str; 3] = ["toLocaleString", "toString", "valueOf"];

#[expect(
    clippy::disallowed_types,
    reason = "serde_json objects are keyed by std strings"
)]
type JsonMap = Map<std::string::String, Value>;

type Boxed<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Which optional parts of a shape to load. Flags, text, constituents and
/// constraints are always included; each detail costs extra checker requests.
pub(in crate::linter) mod detail {
    /// Load a shape at all (flags, text, constituents, constraint).
    pub(in crate::linter) const STRUCTURE: u8 = 1 << 7;
    /// The type's symbol: name, flags, default-library origin, value
    /// declaration kind.
    pub(in crate::linter) const SYMBOL: u8 = 1 << 0;
    /// Array element and tuple element types.
    pub(in crate::linter) const ELEMENTS: u8 = 1 << 1;
    /// Names of the transitive base types.
    pub(in crate::linter) const BASES: u8 = 1 << 2;
    /// Whether the type has call signatures.
    pub(in crate::linter) const CALLABLE: u8 = 1 << 3;
    /// Construct signatures (own and of the symbol's type), property count,
    /// `[Symbol.iterator]`.
    pub(in crate::linter) const OBJECT_KIND: u8 = 1 << 4;
    /// Where `toString`, `toLocaleString` and `valueOf` come from.
    pub(in crate::linter) const TO_STRING: u8 = 1 << 5;
}

/// Builds and caches shape trees for one snapshot.
pub(super) struct Shapes<'a> {
    session: &'a CheckerMemo<'a>,
    file: &'a str,
    /// Shapes by type, with the depth and details they were built with.
    cache: FxHashMap<TypeHandle, (u8, u8, Value)>,
    object_members: Option<FxHashMap<&'static str, Vec<NodeHandle>>>,
    /// unbound-method facts by receiver type and member name.
    member_cache: FxHashMap<(TypeHandle, vize_l0::String), Value>,
    /// `checkIfMethod` inputs by property symbol.
    method_cache: FxHashMap<corsa::api::SymbolHandle, Value>,
    /// Receiver symbol facts by type.
    receiver_cache: FxHashMap<TypeHandle, JsonMap>,
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
        }
    }

    pub(super) async fn shape(&mut self, ty: &TypeResponse, details: u8) -> Value {
        self.build(ty.clone(), 0, details).await
    }

    fn build(&mut self, ty: TypeResponse, depth: u8, details: u8) -> Boxed<'_, Value> {
        Box::pin(async move {
            if let Some((built_at, built_with, cached)) = self.cache.get(&ty.id)
                && *built_at <= depth
                && built_with & details == details
            {
                return cached.clone();
            }
            let shape = self.compute(&ty, depth, details).await;
            self.cache
                .insert(ty.id.clone(), (depth, details, shape.clone()));
            shape
        })
    }

    async fn compute(&mut self, ty: &TypeResponse, depth: u8, details: u8) -> Value {
        let flags = ty.flags;
        let object_flags = ty.object_flags.unwrap_or(0);
        let mut out = Map::new();
        out.insert("id".into(), json!(ty.id.as_str()));
        out.insert("flags".into(), json!(flags));
        out.insert("objectFlags".into(), json!(object_flags));
        out.insert("text".into(), json!(self.text(ty).await));
        if depth >= MAX_DEPTH {
            return Value::Object(out);
        }
        if flags & (UNION | INTERSECTION) != 0 {
            let parts = self.session.constituents(ty).await;
            let mut shapes = Vec::with_capacity(parts.len());
            for part in parts {
                shapes.push(self.build(part, depth + 1, details).await);
            }
            out.insert("types".into(), Value::Array(shapes));
        }
        if flags & GENERIC != 0 {
            let constraint = self
                .session
                .base_constraint(ty)
                .await
                .filter(|constraint| constraint.id != ty.id);
            if let Some(constraint) = constraint {
                let shape = self.build(constraint, depth + 1, details).await;
                out.insert("constraint".into(), shape);
            }
        }
        if flags & OBJECT != 0 {
            self.object_facts(ty, depth, details, &mut out).await;
        }
        if flags & (OBJECT | NON_PRIMITIVE) != 0 && details & detail::TO_STRING != 0 {
            out.insert("toString".into(), self.object_coercion(ty).await);
        }
        Value::Object(out)
    }

    async fn text(&self, ty: &TypeResponse) -> vize_l0::String {
        self.session
            .type_texts(ty)
            .await
            .into_iter()
            .next()
            .unwrap_or_default()
    }

    async fn object_facts(&mut self, ty: &TypeResponse, depth: u8, details: u8, out: &mut JsonMap) {
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let object_flags = ty.object_flags.unwrap_or(0);
        if ty.symbol.is_some()
            && details & (detail::SYMBOL | detail::OBJECT_KIND) != 0
            && let Ok(Some(symbol)) = client
                .get_symbol_of_type_in_project(snapshot.clone(), project.clone(), ty.id.clone())
                .await
        {
            out.insert("symbol".into(), json!(symbol.name));
            out.insert("symbolFlags".into(), json!(symbol.flags));
            if let Some(kind) = symbol.value_declaration.as_ref().and_then(declaration_kind) {
                out.insert("valueDeclarationKind".into(), json!(kind));
            }
            let symbol_type = if details & detail::OBJECT_KIND != 0 {
                client
                    .get_type_of_symbol(snapshot.clone(), project.clone(), symbol.id.clone())
                    .await
                    .ok()
                    .flatten()
            } else {
                None
            };
            if let Some(symbol_type) = symbol_type {
                let constructable = !client
                    .get_signatures_of_type(snapshot.clone(), project.clone(), symbol_type.id, 1)
                    .await
                    .unwrap_or_default()
                    .is_empty();
                out.insert("symbolConstructable".into(), json!(constructable));
            }
            out.insert(
                "lib".into(),
                json!(
                    !symbol.declarations.is_empty()
                        && symbol.declarations.iter().all(is_default_library)
                ),
            );
        }
        if details & detail::CALLABLE != 0 {
            let signatures = client
                .get_signatures_of_type(snapshot.clone(), project.clone(), ty.id.clone(), 0)
                .await
                .unwrap_or_default();
            out.insert("callable".into(), json!(!signatures.is_empty()));
        }
        if details & detail::OBJECT_KIND != 0 {
            self.object_kind(ty, out).await;
        }
        if object_flags & REFERENCE != 0 && details & detail::ELEMENTS != 0 {
            self.elements(ty, depth, details, out).await;
        }
        if object_flags & (CLASS | INTERFACE | REFERENCE) != 0 && details & detail::BASES != 0 {
            let mut bases = Vec::new();
            self.collect_bases(ty.id.clone(), 0, &mut bases).await;
            out.insert("bases".into(), json!(bases));
        }
    }

    /// Construct signatures, property count and iterability.
    async fn object_kind(&self, ty: &TypeResponse, out: &mut JsonMap) {
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let constructable = !client
            .get_signatures_of_type(snapshot.clone(), project.clone(), ty.id.clone(), 1)
            .await
            .unwrap_or_default()
            .is_empty();
        out.insert("constructable".into(), json!(constructable));
        let properties = self
            .session
            .get_properties_of_type(ty.id.clone())
            .await
            .unwrap_or_default();
        out.insert("propertyCount".into(), json!(properties.len()));
        out.insert(
            "iterable".into(),
            json!(
                properties
                    .iter()
                    .any(|property| property.name.starts_with("__@iterator@"))
            ),
        );
    }

    /// Array element or tuple element types of a type reference.
    async fn elements(&mut self, ty: &TypeResponse, depth: u8, details: u8, out: &mut JsonMap) {
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let is_array = client
            .is_array_type(snapshot.clone(), project.clone(), ty.id.clone())
            .await
            .unwrap_or(false);
        // Tuple references carry their element flags; `isTupleType` is not
        // served by every runtime.
        let is_tuple = !is_array
            && (!ty.element_flags.is_empty()
                || ty.fixed_length.is_some()
                || client
                    .is_tuple_type(snapshot.clone(), project.clone(), ty.id.clone())
                    .await
                    .unwrap_or(false));
        if is_array || is_tuple {
            let arguments = client
                .get_type_arguments(snapshot.clone(), project.clone(), ty.id.clone())
                .await
                .unwrap_or_default();
            let mut shapes = Vec::with_capacity(arguments.len());
            for argument in arguments {
                shapes.push(self.build(argument, depth + 1, details).await);
            }
            if is_array {
                if let Some(element) = shapes.into_iter().next() {
                    out.insert("arrayElement".into(), element);
                }
            } else {
                out.insert("tupleElements".into(), Value::Array(shapes));
            }
        }
    }

    fn collect_bases<'s>(
        &'s self,
        ty: TypeHandle,
        depth: u8,
        out: &'s mut Vec<vize_l0::String>,
    ) -> Boxed<'s, ()> {
        Box::pin(async move {
            if depth >= BASE_DEPTH {
                return;
            }
            let bases = self
                .session
                .client()
                .get_base_types(
                    self.session.snapshot().handle.clone(),
                    self.session.project_handle(),
                    ty,
                )
                .await
                .unwrap_or_default();
            for base in bases {
                let text = self.text(&base).await;
                if out.contains(&text) {
                    continue;
                }
                out.push(text);
                self.collect_bases(base.id.clone(), depth + 1, out).await;
            }
        })
    }

    /// typescript-eslint's `isToStringLikeFromObject`: `false` when the type
    /// declares its own coercion method (or `[Symbol.toPrimitive]`), `true`
    /// when every coercion method comes from `interface Object`, `null` when
    /// none resolves.
    async fn object_coercion(&mut self, ty: &TypeResponse) -> Value {
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let properties = self
            .session
            .get_properties_of_type(ty.id.clone())
            .await
            .unwrap_or_default();
        let declares_to_primitive = properties.iter().any(|property| {
            property.name.contains("toPrimitive")
                && property
                    .value_declaration
                    .as_ref()
                    .and_then(declaration_kind)
                    .is_some_and(|kind| kind == METHOD_SIGNATURE)
        });
        if declares_to_primitive {
            return json!(false);
        }
        let Some(object_members) = self.object_members().await else {
            return Value::Null;
        };
        let mut found_on_object = false;
        for name in COERCION_METHODS {
            let Ok(Some(candidate)) = client
                .get_property_of_type(snapshot.clone(), project.clone(), ty.id.clone(), name)
                .await
            else {
                continue;
            };
            if candidate.declarations.is_empty() {
                continue;
            }
            let on_object = object_members.get(name).is_some_and(|declarations| {
                candidate
                    .declarations
                    .iter()
                    .all(|declaration| declarations.contains(declaration))
            });
            if !on_object {
                return json!(false);
            }
            found_on_object = true;
        }
        if found_on_object {
            json!(true)
        } else {
            Value::Null
        }
    }

    /// Declarations of `toString`, `toLocaleString` and `valueOf` on the
    /// global `interface Object`.
    async fn object_members(&mut self) -> Option<FxHashMap<&'static str, Vec<NodeHandle>>> {
        if let Some(members) = &self.object_members {
            return Some(members.clone());
        }
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let symbol = client
            .resolve_name(
                snapshot.clone(),
                project.clone(),
                "Object",
                SYMBOL_INTERFACE,
                None,
                Some(DocumentIdentifier::from(self.file)),
                Some(0),
                Some(false),
            )
            .await
            .ok()
            .flatten()?;
        let object = client
            .get_declared_type_of_symbol(snapshot.clone(), project.clone(), symbol.id)
            .await
            .ok()
            .flatten()?;
        let mut members = FxHashMap::default();
        for name in COERCION_METHODS {
            let declarations = client
                .get_property_of_type(snapshot.clone(), project.clone(), object.id.clone(), name)
                .await
                .ok()
                .flatten()
                .map(|symbol| symbol.declarations)
                .unwrap_or_default();
            members.insert(name, declarations);
        }
        self.object_members = Some(members.clone());
        Some(members)
    }
}
