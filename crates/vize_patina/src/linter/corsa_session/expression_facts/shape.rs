//! Structural checker facts for one expression type.
//!
//! typescript-eslint rules inspect `TypeFlags`, union and intersection
//! constituents, type-parameter constraints, array elements, base types and
//! where `toString` comes from. Rendered type text cannot answer those
//! questions, so the host sends them as a small JSON tree (`__typeShape`).
use corsa::api::{DocumentIdentifier, NodeHandle, ProjectSession, TypeHandle, TypeResponse};
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
const FUNCTION_EXPRESSION: u16 = 220;
const PROPERTY_ASSIGNMENT: u16 = 304;

/// Nesting depth of the shape tree. Deep enough for `T[]` inside a union
/// inside a constraint.
const MAX_DEPTH: u8 = 4;
const BASE_DEPTH: u8 = 6;
const COERCION_METHODS: [&str; 3] = ["toLocaleString", "toString", "valueOf"];

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
    session: &'a ProjectSession,
    file: &'a str,
    /// Shapes by type, with the depth and details they were built with.
    cache: FxHashMap<TypeHandle, (u8, u8, Value)>,
    object_members: Option<FxHashMap<&'static str, Vec<NodeHandle>>>,
    /// unbound-method facts by receiver type and member name.
    member_cache: FxHashMap<(TypeHandle, std::string::String), Value>,
    /// `checkIfMethod` inputs by property symbol.
    method_cache: FxHashMap<corsa::api::SymbolHandle, Value>,
    /// Receiver symbol facts by type.
    receiver_cache: FxHashMap<TypeHandle, Map<std::string::String, Value>>,
}

impl<'a> Shapes<'a> {
    pub(super) fn new(session: &'a ProjectSession, file: &'a str) -> Self {
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
            if let Some((built_at, built_with, cached)) = self.cache.get(&ty.id) {
                if *built_at <= depth && built_with & details == details {
                    return cached.clone();
                }
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
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        if flags & (UNION | INTERSECTION) != 0 {
            let parts = client
                .get_types_of_type_in_project(snapshot.clone(), project.clone(), ty.id.clone())
                .await
                .unwrap_or_default();
            let mut shapes = Vec::with_capacity(parts.len());
            for part in parts {
                shapes.push(self.build(part, depth + 1, details).await);
            }
            out.insert("types".into(), Value::Array(shapes));
        }
        if flags & GENERIC != 0 {
            let constraint = client
                .get_base_constraint_of_type(snapshot.clone(), project.clone(), ty.id.clone())
                .await
                .ok()
                .flatten()
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
            out.insert("toString".into(), self.to_string_like(ty).await);
        }
        Value::Object(out)
    }

    async fn text(&self, ty: &TypeResponse) -> std::string::String {
        if let Some(text) = ty.texts.first() {
            return text.clone();
        }
        self.session
            .render_type_texts(ty)
            .await
            .ok()
            .and_then(|texts| texts.into_iter().next())
            .map(|text| text.as_str().to_owned())
            .unwrap_or_default()
    }

    async fn object_facts(
        &mut self,
        ty: &TypeResponse,
        depth: u8,
        details: u8,
        out: &mut Map<std::string::String, Value>,
    ) {
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let object_flags = ty.object_flags.unwrap_or(0);
        if ty.symbol.is_some() && details & (detail::SYMBOL | detail::OBJECT_KIND) != 0 {
            if let Ok(Some(symbol)) = client
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
                        .get_signatures_of_type(
                            snapshot.clone(),
                            project.clone(),
                            symbol_type.id,
                            1,
                        )
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
    async fn object_kind(&self, ty: &TypeResponse, out: &mut Map<std::string::String, Value>) {
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
    async fn elements(
        &mut self,
        ty: &TypeResponse,
        depth: u8,
        details: u8,
        out: &mut Map<std::string::String, Value>,
    ) {
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
        out: &'s mut Vec<std::string::String>,
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
    async fn to_string_like(&mut self, ty: &TypeResponse) -> Value {
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

impl Shapes<'_> {
    /// unbound-method facts for `names` on this receiver type: the receiver's
    /// symbol, and per union/intersection constituent how each property is
    /// declared (`checkIfMethod`).
    pub(super) async fn member_methods(
        &mut self,
        ty: &TypeResponse,
        names: &[std::string::String],
    ) -> Value {
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let mut out = match self.receiver_cache.get(&ty.id) {
            Some(receiver) => receiver.clone(),
            None => {
                let receiver = self.receiver(ty).await;
                self.receiver_cache.insert(ty.id.clone(), receiver.clone());
                receiver
            }
        };
        let mut members = Map::new();
        let mut parts = None;
        for name in names {
            let key = (ty.id.clone(), name.clone());
            if let Some(cached) = self.member_cache.get(&key) {
                members.insert(name.clone(), cached.clone());
                continue;
            }
            if parts.is_none() {
                parts = Some(self.constituents(ty).await);
            }
            let parts = parts.as_deref().unwrap_or_default();
            let mut infos = Vec::with_capacity(parts.len());
            let mut property_lib = None;
            for part in parts {
                let property = client
                    .get_property_of_type(snapshot.clone(), project.clone(), part.id.clone(), name)
                    .await
                    .ok()
                    .flatten();
                let Some(property) = property else {
                    infos.push(Value::Null);
                    continue;
                };
                if property_lib.is_none() {
                    property_lib = Some(
                        !property.declarations.is_empty()
                            && property.declarations.iter().all(is_default_library),
                    );
                }
                let info = match self.method_cache.get(&property.id) {
                    Some(info) => info.clone(),
                    None => {
                        let info = self.method_info(&property).await;
                        self.method_cache.insert(property.id.clone(), info.clone());
                        info
                    }
                };
                infos.push(info);
            }
            let member = json!({ "parts": infos, "propertyLib": property_lib });
            self.member_cache.insert(key, member.clone());
            members.insert(name.clone(), member);
        }
        out.insert("members".into(), Value::Object(members));
        Value::Object(out)
    }

    /// The receiver's symbol and base type names (`isBuiltinSymbolLike`).
    async fn receiver(&self, ty: &TypeResponse) -> Map<std::string::String, Value> {
        let mut out = Map::new();
        if ty.symbol.is_some() {
            if let Ok(Some(symbol)) = self
                .session
                .client()
                .get_symbol_of_type_in_project(
                    self.session.snapshot().handle.clone(),
                    self.session.project_handle(),
                    ty.id.clone(),
                )
                .await
            {
                // `typeof Foo` for `class Foo extends Array`: the class's
                // instance bases count too (`isBuiltinSymbolLike`).
                if symbol.flags & SYMBOL_CLASS != 0 {
                    if let Ok(Some(declared)) = self
                        .session
                        .client()
                        .get_declared_type_of_symbol(
                            self.session.snapshot().handle.clone(),
                            self.session.project_handle(),
                            symbol.id.clone(),
                        )
                        .await
                    {
                        let mut bases = Vec::new();
                        self.collect_bases(declared.id, 0, &mut bases).await;
                        out.insert("receiverClassBases".into(), json!(bases));
                    }
                }
                out.insert("receiverSymbol".into(), json!(symbol.name));
                out.insert(
                    "receiverLib".into(),
                    json!(
                        !symbol.declarations.is_empty()
                            && symbol.declarations.iter().all(is_default_library)
                    ),
                );
            }
        }
        if ty.flags & OBJECT != 0 {
            let mut bases = Vec::new();
            self.collect_bases(ty.id.clone(), 0, &mut bases).await;
            out.insert("receiverBases".into(), json!(bases));
        }
        out
    }

    /// Union constituents, each flattened into its intersection constituents.
    async fn constituents(&self, ty: &TypeResponse) -> Vec<TypeResponse> {
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let unions = if ty.flags & UNION != 0 {
            client
                .get_types_of_type_in_project(snapshot.clone(), project.clone(), ty.id.clone())
                .await
                .unwrap_or_default()
        } else {
            vec![ty.clone()]
        };
        let mut parts = Vec::new();
        for part in unions {
            if part.flags & INTERSECTION != 0 {
                parts.extend(
                    client
                        .get_types_of_type_in_project(
                            snapshot.clone(),
                            project.clone(),
                            part.id.clone(),
                        )
                        .await
                        .unwrap_or_default(),
                );
            } else {
                parts.push(part);
            }
        }
        parts
    }

    /// typescript-eslint's `checkIfMethod`/`checkMethod` inputs for one
    /// property symbol.
    async fn method_info(&self, property: &corsa::api::SymbolResponse) -> Value {
        let kind = property
            .value_declaration
            .as_ref()
            .and_then(declaration_kind);
        let method_like = matches!(kind, Some(METHOD_SIGNATURE | METHOD_DECLARATION));
        let property_like = matches!(kind, Some(PROPERTY_DECLARATION | PROPERTY_ASSIGNMENT));
        if !method_like && !property_like {
            return json!({ "isMethod": false });
        }
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let Some(mut property_type) = client
            .get_type_of_symbol(snapshot.clone(), project.clone(), property.id.clone())
            .await
            .ok()
            .flatten()
        else {
            return json!({ "isMethod": method_like });
        };
        // An optional method's type is `(() => void) | undefined`.
        if property_type.flags & UNION != 0 {
            if let Ok(Some(non_nullable)) = client
                .get_non_nullable_type(snapshot.clone(), project.clone(), property_type.id.clone())
                .await
            {
                property_type = non_nullable;
            }
        }
        let signatures = client
            .get_signatures_of_type(snapshot.clone(), project.clone(), property_type.id, 0)
            .await
            .unwrap_or_default();
        let Some(signature) = signatures.into_iter().next() else {
            return json!({ "isMethod": method_like });
        };
        // A property is a method only when initialized with a `function`
        // expression; arrow functions keep their lexical `this`.
        let is_method = method_like
            || signature.declaration.as_ref().and_then(declaration_kind)
                == Some(FUNCTION_EXPRESSION);
        let first_param_is_this =
            signature.this_parameter.is_some() || signature.this_parameter_symbol.is_some();
        let this_arg_is_void = if signature
            .this_parameter_type_texts
            .iter()
            .any(|text| text.trim() == "void")
        {
            true
        } else if let Some(this_parameter) = signature.this_parameter.clone() {
            client
                .get_type_of_symbol(
                    self.session.snapshot().handle.clone(),
                    self.session.project_handle(),
                    this_parameter,
                )
                .await
                .ok()
                .flatten()
                .is_some_and(|this_type| this_type.flags == VOID)
        } else {
            false
        };
        json!({
            "isMethod": is_method,
            "firstParamIsThis": first_param_is_this,
            "thisArgIsVoid": this_arg_is_void,
            "isStatic": false,
        })
    }
}

fn declaration_kind(handle: &NodeHandle) -> Option<u16> {
    if let Ok(parsed) = handle.parse() {
        return Some(parsed.kind);
    }
    // Compact stable-runtime handles: `<node id>.<kind>.<path>`.
    handle.as_str().split('.').nth(1)?.parse().ok()
}

/// Whether a declaration lives in one of TypeScript's bundled `lib.*.d.ts`
/// files.
fn is_default_library(handle: &NodeHandle) -> bool {
    handle.declaring_path().is_some_and(|path| {
        let name = path.rsplit('/').next().unwrap_or_default();
        name.starts_with("lib.") && name.ends_with(".d.ts")
    })
}
