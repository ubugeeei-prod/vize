//! unbound-method member facts: the receiver's symbol and how each property is declared (`checkIfMethod`).
use super::{
    FUNCTION_EXPRESSION, INTERSECTION, JsonMap, METHOD_DECLARATION, METHOD_SIGNATURE, OBJECT,
    PROPERTY_ASSIGNMENT, PROPERTY_DECLARATION, SYMBOL_CLASS, Shapes, UNION, VOID,
};
use corsa::api::{NodeHandle, TypeResponse};
use serde_json::{Map, Value, json};

impl Shapes<'_> {
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
                json!(
                    !symbol.declarations.is_empty()
                        && symbol.declarations.iter().all(is_default_library)
                ),
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

pub(super) fn declaration_kind(handle: &NodeHandle) -> Option<u16> {
    if let Ok(parsed) = handle.parse() {
        return Some(parsed.kind);
    }
    // Compact stable-runtime handles: `<node id>.<kind>.<path>`.
    handle.as_str().split('.').nth(1)?.parse().ok()
}

/// Whether a declaration lives in one of TypeScript's bundled `lib.*.d.ts`
/// files.
pub(super) fn is_default_library(handle: &NodeHandle) -> bool {
    handle.declaring_path().is_some_and(|path| {
        let name = path.rsplit('/').next().unwrap_or_default();
        name.starts_with("lib.") && name.ends_with(".d.ts")
    })
}

#[cfg(test)]
mod tests {
    use corsa::api::NodeHandle;

    use super::{FUNCTION_EXPRESSION, declaration_kind};

    #[test]
    fn typescript_seven_distinguishes_function_expressions_from_arrows() {
        let function = NodeHandle::from("42.219./workspace/composables.ts");
        let arrow = NodeHandle::from("43.220./workspace/composables.ts");
        assert_eq!(declaration_kind(&function), Some(FUNCTION_EXPRESSION));
        assert_ne!(declaration_kind(&arrow), Some(FUNCTION_EXPRESSION));
    }
}
