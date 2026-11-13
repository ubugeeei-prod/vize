//! Memoized property, signature, index and shape lookups of the relation checker.
use super::{CALL, Checker, flags};
use corsa::api::{SignatureResponse, SymbolResponse, TypeResponse};

impl Checker<'_> {
    pub(in super::super) async fn properties(&mut self, ty: &TypeResponse) -> Vec<SymbolResponse> {
        if let Some(properties) = self.properties.get(&ty.id) {
            return properties.clone();
        }
        let properties = self
            .session
            .get_properties_of_type(ty.id.clone())
            .await
            .unwrap_or_default();
        self.properties.insert(ty.id.clone(), properties.clone());
        properties
    }

    pub(in super::super) async fn property(
        &mut self,
        ty: &TypeResponse,
        name: &str,
    ) -> Option<SymbolResponse> {
        self.session
            .client()
            .get_property_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
                name,
            )
            .await
            .ok()
            .flatten()
    }

    pub(in super::super) async fn type_of_symbol(
        &mut self,
        symbol: &SymbolResponse,
    ) -> Option<TypeResponse> {
        self.session
            .get_type_of_symbol(symbol.id.clone())
            .await
            .ok()
            .flatten()
    }

    pub(in super::super) async fn signatures(
        &mut self,
        ty: &TypeResponse,
        kind: i32,
    ) -> Vec<SignatureResponse> {
        let key = (ty.id.clone(), kind);
        if let Some(signatures) = self.signatures.get(&key) {
            return signatures.clone();
        }
        let signatures = self
            .session
            .get_signatures_of_type(ty.id.clone(), kind)
            .await
            .unwrap_or_default();
        self.signatures.insert(key, signatures.clone());
        signatures
    }

    pub(in super::super) async fn return_type(
        &mut self,
        signature: &SignatureResponse,
    ) -> Option<TypeResponse> {
        self.session
            .get_return_type_of_signature(signature.id.clone())
            .await
            .ok()
            .flatten()
    }

    /// Types of a signature's parameters, in order.
    pub(in super::super) async fn parameter_types(
        &mut self,
        signature: &SignatureResponse,
    ) -> Vec<TypeResponse> {
        let mut types = Vec::new();
        for parameter in &signature.parameters {
            if let Ok(Some(ty)) = self.session.get_type_of_symbol(parameter.clone()).await {
                types.push(ty);
            }
        }
        types
    }

    pub(in super::super) async fn index_info_count(&mut self, ty: &TypeResponse) -> usize {
        self.session
            .client()
            .get_index_infos_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .map(|infos| infos.len())
            .unwrap_or(0)
    }

    /// `aliasTypeArguments ?? (isTypeReference(type) ? checker.getTypeArguments(type) : [])`.
    pub(in super::super) async fn type_arguments(
        &mut self,
        ty: &TypeResponse,
    ) -> Vec<TypeResponse> {
        let snapshot = self.session.snapshot().handle.clone();
        let project = self.session.project_handle();
        let alias = self
            .session
            .client()
            .get_alias_type_arguments_of_type(snapshot.clone(), project.clone(), ty.id.clone())
            .await
            .unwrap_or_default();
        if !alias.is_empty() || ty.flags & flags::OBJECT == 0 {
            return alias;
        }
        self.session
            .client()
            .get_type_arguments(snapshot, project, ty.id.clone())
            .await
            .unwrap_or_default()
    }

    /// The uncast type of an immediately invoked function: the return type of
    /// its callee's first call signature, or `void` when the callee declares
    /// no return type and returns `undefined`.
    pub(in super::super) async fn iife_return(
        &mut self,
        callee: &TypeResponse,
        annotated: bool,
    ) -> Option<TypeResponse> {
        let signature = self.signatures(callee, CALL).await.into_iter().next()?;
        let returned = self.return_type(&signature).await?;
        if !annotated && self.union_flags(&returned).await & flags::UNDEFINED != 0 {
            return self
                .session
                .client()
                .get_void_type(
                    self.session.snapshot().handle.clone(),
                    self.session.project_handle(),
                )
                .await
                .ok();
        }
        Some(returned)
    }
}
