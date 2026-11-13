//! Object types and signatures in the `visitType` walk.
use super::{Boxed, CLASS, INTERFACE, MAPPED, NUMBER, STRING, Walk};
use corsa::api::{SignatureResponse, SymbolHandle, TypeResponse};

impl Walk<'_, '_> {
    /// `isObjectType(type)`: properties, index signatures, then call and
    /// construct signatures.
    pub(super) async fn object(&mut self, ty: &TypeResponse, object_flags: u32) {
        if object_flags & MAPPED != 0 && self.mapped(ty).await {
            return;
        }
        if object_flags & (CLASS | INTERFACE) != 0
            && !ty
                .outer_type_parameters
                .iter()
                .chain(&ty.local_type_parameters)
                .any(|parameter| self.own.contains(parameter))
        {
            // A class or interface declared outside the owner cannot name
            // the owner's type parameters.
            return;
        }
        if self.visited_members.insert(ty.id.clone()) {
            if !self.spend() {
                return;
            }
            let properties = self.memo.get_properties_of_type(ty.id.clone()).await;
            let Some(properties) = self.give_up(properties) else {
                return;
            };
            let symbols = properties
                .into_iter()
                .map(|property| property.id)
                .collect::<Vec<_>>();
            for property in self.symbol_types(symbols).await {
                self.visit(property, false, false).await;
            }
        }
        if !self.spend() {
            return;
        }
        let infos = self
            .memo
            .client()
            .get_index_infos_of_type(
                self.memo.snapshot().handle.clone(),
                self.memo.project_handle(),
                ty.id.clone(),
            )
            .await;
        for info in self.give_up(infos).unwrap_or_default() {
            if info.key_type.flags & (STRING | NUMBER) != 0 {
                self.visit(info.value_type, true, false).await;
            }
        }
        for kind in [0, 1] {
            if !self.spend() {
                return;
            }
            let signatures = self.memo.get_signatures_of_type(ty.id.clone(), kind).await;
            for signature in self.give_up(signatures).unwrap_or_default() {
                self.signature(signature).await;
            }
        }
    }

    /// A mapped type. The API exposes neither its type parameter nor its
    /// template, name or constraint types. With resolved keys, its
    /// properties are walked as any object's. A generic one (no properties
    /// yet) counts each own parameter its rendering names, the uses
    /// upstream's visit of those parts finds, as long as nothing in the
    /// rendering could hide a repeated use: an elided part, or a generic
    /// whose arguments upstream counts twice. Returns whether it was handled.
    async fn mapped(&mut self, ty: &TypeResponse) -> bool {
        if !self.spend() {
            return true;
        }
        let properties = self.memo.get_properties_of_type(ty.id.clone()).await;
        let Some(properties) = self.give_up(properties) else {
            return true;
        };
        if !properties.is_empty() {
            return false;
        }
        let texts = self.memo.type_texts(ty).await;
        let Some(text) = texts.first() else {
            self.uncertain = true;
            return true;
        };
        if text.contains("...") || text.contains('<') {
            self.uncertain = true;
            return true;
        }
        for (name, count) in self.names.iter().zip(self.counts.iter_mut()) {
            *count += mentions(text, name);
        }
        true
    }

    pub(super) async fn symbol_types(&mut self, symbols: Vec<SymbolHandle>) -> Vec<TypeResponse> {
        if symbols.is_empty() || !self.spend() {
            return Vec::new();
        }
        let types = self.memo.get_types_of_symbols(symbols).await;
        self.give_up(types)
            .unwrap_or_default()
            .into_iter()
            .flatten()
            .collect()
    }

    /// `visitSignature`: `this`, parameters, type parameters, then the
    /// predicate type or the return type.
    pub(in super::super) fn signature(&mut self, signature: SignatureResponse) -> Boxed<'_, ()> {
        Box::pin(async move {
            // The signature's own symbol handles are mentions; the
            // parameter getters return handles this connection can resolve.
            let client = self.memo.client();
            let snapshot = self.memo.snapshot().handle.clone();
            let project = self.memo.project_handle();
            let mut symbols = Vec::new();
            if signature.this_parameter.is_some() {
                if !self.spend() {
                    return;
                }
                let this = client
                    .get_this_parameter_of_signature(
                        snapshot.clone(),
                        project.clone(),
                        signature.id.clone(),
                    )
                    .await;
                symbols.extend(self.give_up(this).flatten().map(|this| this.id));
            }
            if !signature.parameters.is_empty() {
                if !self.spend() {
                    return;
                }
                let parameters = client
                    .get_parameters_of_signature(
                        snapshot.clone(),
                        project.clone(),
                        signature.id.clone(),
                    )
                    .await;
                symbols.extend(
                    self.give_up(parameters)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|parameter| parameter.id),
                );
            }
            for parameter in self.symbol_types(symbols).await {
                self.visit(parameter, false, false).await;
            }
            for parameter in signature.type_parameters {
                if self.done() || self.seen_too_often(&parameter) {
                    continue;
                }
                self.type_parameter(parameter, false).await;
            }
            if self.done() || !self.spend() {
                return;
            }
            // A runtime without the predicate endpoint reads the return type,
            // which is the predicate's `boolean` there.
            let predicate = client
                .get_type_predicate_of_signature(
                    snapshot.clone(),
                    project.clone(),
                    signature.id.clone(),
                )
                .await
                .ok()
                .flatten()
                .and_then(|predicate| predicate.r#type);
            if let Some(predicate) = predicate {
                self.visit(predicate, false, true).await;
                return;
            }
            if !self.spend() {
                return;
            }
            let returned = client
                .get_return_type_of_signature(snapshot, project, signature.id)
                .await;
            if let Some(Some(returned)) = self.give_up(returned) {
                self.visit(returned, false, true).await;
            }
        })
    }
}

/// How often `name` appears as a whole identifier in `text`.
fn mentions(text: &str, name: &str) -> u32 {
    let word = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$';
    let bytes = text.as_bytes();
    let mut found = 0;
    for (start, _) in text.match_indices(name) {
        let before = start.checked_sub(1).and_then(|at| bytes.get(at).copied());
        let after = bytes.get(start + name.len()).copied();
        if !before.is_some_and(word) && !after.is_some_and(word) {
            found += 1;
        }
    }
    found
}
