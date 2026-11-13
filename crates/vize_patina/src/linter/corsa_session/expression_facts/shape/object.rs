//! Object facts of a type tree node: its symbol, signatures, base types,
//! where its coercion methods come from, and its numeric index.
use super::super::super::checker_memo::syntax_kind;
use super::members::is_default_library;
use super::{
    BASE_DEPTH, Boxed, CLASS, COERCION_METHODS, INTERFACE, JsonMap, NUMBER, NUMBER_LIKE, REFERENCE,
    SYMBOL_INTERFACE, Shapes, UNION, detail,
};
use corsa::api::{DocumentIdentifier, NodeHandle, TypeHandle, TypeResponse};
use serde_json::{Value, json};
use vize_l0::FxHashMap;

impl Shapes<'_> {
    /// Symbol, signature, base-type and coercion facts of an object type.
    pub(super) async fn object_facts(
        &mut self,
        ty: &TypeResponse,
        details: u16,
        out: &mut JsonMap,
    ) {
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

    pub(super) fn collect_bases<'s>(
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
    pub(super) async fn object_coercion(&mut self, ty: &TypeResponse) -> Value {
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
                    .and_then(|handle| self.session.syntax_kind(handle))
                    .is_some_and(|kind| kind == syntax_kind::METHOD_SIGNATURE)
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

    /// A `number` index signature, and a `length` property of a number-like
    /// type: `isArrayLikeType`'s test for `for...in` and `delete`.
    pub(super) async fn number_index(&self, ty: &TypeResponse, out: &mut JsonMap) {
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let client = self.session.client();
        let index = client
            .get_index_infos_of_type(snapshot.clone(), project.clone(), ty.id.clone())
            .await
            .unwrap_or_default()
            .iter()
            .any(|info| info.key_type.flags & NUMBER != 0);
        out.insert("numberIndex".into(), json!(index));
        let length = match client
            .get_property_of_type(snapshot.clone(), project.clone(), ty.id.clone(), "length")
            .await
        {
            Ok(Some(length)) => client
                .get_type_of_symbol(snapshot, project, length.id)
                .await
                .ok()
                .flatten(),
            _ => None,
        };
        let length_flags = match length {
            Some(length) if length.flags & UNION != 0 => self
                .session
                .constituents(&length)
                .await
                .iter()
                .fold(length.flags, |all, part| all | part.flags),
            Some(length) => length.flags,
            None => 0,
        };
        out.insert(
            "numberLength".into(),
            json!(length_flags & NUMBER_LIKE != 0),
        );
    }
}
