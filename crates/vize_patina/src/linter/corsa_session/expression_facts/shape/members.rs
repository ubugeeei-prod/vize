//! unbound-method member facts: the receiver's symbol and how each property is declared (`checkIfMethod`).
use super::{
    FUNCTION_EXPRESSION, INTERSECTION, JsonMap, METHOD_DECLARATION, METHOD_SIGNATURE, OBJECT,
    PROPERTY_ASSIGNMENT, PROPERTY_DECLARATION, SYMBOL_CLASS, Shapes, UNION, VOID,
};
use corsa::api::{NodeHandle, TypeResponse};
use serde_json::{Map, Value, json};

impl Shapes<'_> {
    /// Method facts for a computed access whose key type resolves to one or
    /// more string/number literals. This reuses the receiver and method caches
    /// used by static member access.
    pub(in super::super) async fn computed_method(
        &mut self,
        receiver: &TypeResponse,
        key: &TypeResponse,
    ) -> Option<Value> {
        let mut names = Vec::new();
        let key_parts = if key.flags & UNION != 0 {
            self.session.constituents(key).await
        } else {
            vec![key.clone()]
        };
        for part in key_parts {
            let name = match &part.value {
                Some(Value::String(value)) => Some(vize_l0::String::from(value.as_str())),
                Some(Value::Number(value)) => Some(vize_l0::cstr!("{value}")),
                _ => None,
            };
            if let Some(name) = name
                && !names.contains(&name)
            {
                names.push(name);
            }
        }
        if names.is_empty() {
            return None;
        }
        let facts = self.member_methods(receiver, &names).await;
        let members = facts.get("members")?.as_object()?;
        let mut safe_method = None;
        let mut static_method = None;
        for name in &names {
            let parts = members.get(name.as_str())?.get("parts")?.as_array()?;
            for part in parts.iter().filter_map(Value::as_object) {
                if part.get("isMethod").and_then(Value::as_bool) != Some(true) {
                    continue;
                }
                let value = Value::Object(part.clone());
                let this_is_void = part
                    .get("thisArgIsVoid")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let is_static = part
                    .get("isStatic")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if !this_is_void && !is_static {
                    return Some(value);
                }
                if !this_is_void && static_method.is_none() {
                    static_method = Some(value);
                } else if safe_method.is_none() {
                    safe_method = Some(value);
                }
            }
        }
        static_method.or(safe_method)
    }

    /// unbound-method facts for `names` on this receiver type: the receiver's
    /// symbol, and per union/intersection constituent how each property is
    /// declared (`checkIfMethod`).
    pub(in super::super) async fn member_methods(
        &mut self,
        ty: &TypeResponse,
        names: &[vize_l0::String],
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
                members.insert(name.as_str().into(), cached.clone());
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
                    .get_property_of_type(
                        snapshot.clone(),
                        project.clone(),
                        part.id.clone(),
                        name.as_str(),
                    )
                    .await
                    .ok()
                    .flatten();
                let Some(property) = property else {
                    infos.push(Value::Null);
                    continue;
                };
                // `isSymbolFromDefaultLibrary`: a declaration merged into a
                // library member elsewhere does not make the member the user's.
                property_lib = Some(
                    property_lib.unwrap_or(false)
                        || declared_in_default_library(&property.declarations),
                );
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
            if property_lib == Some(true) && is_bound_static_side(&out) {
                infos.clear();
                infos.push(json!({ "isMethod": false }));
            }
            let member = json!({ "parts": infos, "propertyLib": property_lib });
            self.member_cache.insert(key, member.clone());
            members.insert(name.as_str().into(), member);
        }
        out.insert("members".into(), Value::Object(members));
        Value::Object(out)
    }

    /// The receiver's symbol and base type names (`isBuiltinSymbolLike`).
    async fn receiver(&self, ty: &TypeResponse) -> JsonMap {
        let mut out = Map::new();
        if ty.symbol.is_some()
            && let Ok(Some(symbol)) = self
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
            if symbol.flags & SYMBOL_CLASS != 0
                && let Ok(Some(declared)) = self
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
            out.insert("receiverSymbol".into(), json!(symbol.name));
            out.insert(
                "receiverLib".into(),
                json!(declared_in_default_library(&symbol.declarations)),
            );
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
        let unions = if ty.flags & UNION != 0 {
            self.session.constituents(ty).await
        } else {
            vec![ty.clone()]
        };
        let mut parts = Vec::new();
        for part in unions {
            if part.flags & INTERSECTION != 0 {
                parts.extend(self.session.constituents(&part).await);
            } else {
                parts.push(part);
            }
        }
        parts
    }

    /// typescript-eslint's `checkIfMethod`/`checkMethod` inputs for one
    /// property symbol.
    pub(in super::super) async fn method_info(
        &self,
        property: &corsa::api::SymbolResponse,
    ) -> Value {
        let kind = property
            .value_declaration
            .as_ref()
            .and_then(declaration_kind);
        let method_like = matches!(kind, Some(METHOD_SIGNATURE | METHOD_DECLARATION));
        let function_property_like = matches!(
            kind,
            Some(PROPERTY_DECLARATION | PROPERTY_ASSIGNMENT | FUNCTION_EXPRESSION)
        );
        if !method_like && !function_property_like {
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
        if property_type.flags & UNION != 0
            && let Ok(Some(non_nullable)) = client
                .get_non_nullable_type(snapshot.clone(), project.clone(), property_type.id.clone())
                .await
        {
            property_type = non_nullable;
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
            || (function_property_like
                && signature.declaration.as_ref().and_then(declaration_kind)
                    == Some(FUNCTION_EXPRESSION));
        let queried_this_parameter = client
            .get_this_parameter_of_signature(
                snapshot.clone(),
                project.clone(),
                signature.id.clone(),
            )
            .await
            .ok()
            .flatten();
        let ordinary_this_parameter = if signature.this_parameter.is_none()
            && signature.this_parameter_symbol.is_none()
            && queried_this_parameter.is_none()
        {
            let parameters = if signature.parameter_symbols.is_empty() {
                client
                    .get_parameters_of_signature(
                        snapshot.clone(),
                        project.clone(),
                        signature.id.clone(),
                    )
                    .await
                    .unwrap_or_default()
            } else {
                signature.parameter_symbols.clone()
            };
            parameters
                .into_iter()
                .find(|parameter| parameter.name == "this")
        } else {
            None
        };
        let this_parameter_symbol = signature
            .this_parameter_symbol
            .as_ref()
            .or(queried_this_parameter.as_ref())
            .or(ordinary_this_parameter.as_ref());
        let this_parameter = signature
            .this_parameter
            .clone()
            .or_else(|| this_parameter_symbol.map(|parameter| parameter.id.clone()));
        let first_param_is_this = this_parameter.is_some();
        let this_arg_is_void = if signature
            .this_parameter_type_texts
            .iter()
            .any(|text| text.trim() == "void")
        {
            true
        } else if let Some(this_parameter) = this_parameter {
            let mut this_type = client
                .get_type_of_symbol(snapshot.clone(), project.clone(), this_parameter.clone())
                .await
                .ok()
                .flatten();
            if let Some(location) =
                this_parameter_symbol.and_then(|parameter| parameter.declarations.first())
                && !this_type.as_ref().is_some_and(|ty| ty.flags & VOID != 0)
            {
                this_type = client
                    .get_type_of_symbol_at_location(
                        snapshot,
                        project,
                        this_parameter,
                        location.clone(),
                    )
                    .await
                    .ok()
                    .flatten()
                    .or(this_type);
            }
            if let Some(this_type) = this_type {
                this_type.flags & VOID != 0
                    || this_type.texts.iter().any(|text| text.trim() == "void")
                    || self
                        .session
                        .render_type_texts(&this_type)
                        .await
                        .unwrap_or_default()
                        .iter()
                        .any(|text| text.trim() == "void")
            } else {
                false
            }
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

pub(super) fn declaration_kind(handle: &NodeHandle) -> Option<u16> {
    if let Ok(parsed) = handle.parse() {
        return Some(parsed.kind);
    }
    // Compact stable-runtime handles: `<node id>.<kind>.<path>`.
    handle.as_str().split('.').nth(1)?.parse().ok()
}

/// typescript-eslint's `isSymbolFromDefaultLibrary`: some declaration of the
/// symbol lives in a default library file. Declaration merging (`declare
/// global { interface Math {} }`) adds declarations; it does not move the
/// library's own ones.
pub(super) fn declared_in_default_library(declarations: &[NodeHandle]) -> bool {
    declarations.iter().any(is_default_library)
}

/// Library static sides whose library members never read `this`, besides the
/// `SUPPORTED_GLOBAL_TYPES` the rule already exempts by type. typescript-eslint
/// exempts `RegExp`'s own members only by the name of the global, from a table
/// of the members its Node has, so `RegExp.escape` depended on the Node version
/// and `window.RegExp.escape` was reported everywhere.
const BOUND_STATIC_SIDES: &[&str] = &["RegExpConstructor"];

/// The receiver is one of [`BOUND_STATIC_SIDES`], declared by the library.
fn is_bound_static_side(receiver: &JsonMap) -> bool {
    receiver.get("receiverLib").and_then(Value::as_bool) == Some(true)
        && receiver
            .get("receiverSymbol")
            .and_then(Value::as_str)
            .is_some_and(|symbol| BOUND_STATIC_SIDES.contains(&symbol))
}

/// Whether a declaration lives in one of TypeScript's bundled `lib.*.d.ts`
/// files.
fn is_default_library(handle: &NodeHandle) -> bool {
    handle.declaring_path().is_some_and(|path| {
        let name = path.rsplit('/').next().unwrap_or_default();
        name.starts_with("lib.") && name.ends_with(".d.ts")
    })
}

#[cfg(test)]
mod tests {
    use corsa::api::NodeHandle;

    use super::{
        FUNCTION_EXPRESSION, METHOD_DECLARATION, METHOD_SIGNATURE, PROPERTY_ASSIGNMENT,
        PROPERTY_DECLARATION, declaration_kind,
    };

    #[test]
    fn typescript_seven_distinguishes_function_expressions_from_arrows() {
        let function = NodeHandle::from("42.219./workspace/composables.ts");
        let arrow = NodeHandle::from("43.220./workspace/composables.ts");
        let property_declaration = NodeHandle::from("44.173./workspace/composables.ts");
        let method_signature = NodeHandle::from("45.174./workspace/composables.ts");
        let method_declaration = NodeHandle::from("46.175./workspace/composables.ts");
        let property = NodeHandle::from("47.303./workspace/composables.ts");
        let shorthand = NodeHandle::from("48.304./workspace/composables.ts");
        assert_eq!(declaration_kind(&function), Some(FUNCTION_EXPRESSION));
        assert_ne!(declaration_kind(&arrow), Some(FUNCTION_EXPRESSION));
        assert_eq!(
            declaration_kind(&property_declaration),
            Some(PROPERTY_DECLARATION)
        );
        assert_eq!(declaration_kind(&method_signature), Some(METHOD_SIGNATURE));
        assert_eq!(
            declaration_kind(&method_declaration),
            Some(METHOD_DECLARATION)
        );
        assert_eq!(declaration_kind(&property), Some(PROPERTY_ASSIGNMENT));
        assert_eq!(declaration_kind(&shorthand), Some(304));
    }
}
