//! Memoized property, signature, index and shape lookups of the relation checker.
use super::super::Shape;
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

    pub(in super::super) async fn is_array(&mut self, ty: &TypeResponse) -> bool {
        self.session
            .client()
            .is_array_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or(false)
    }

    /// `checker.isTupleType`: a reference whose target is a tuple type.
    pub(in super::super) async fn is_tuple(&mut self, ty: &TypeResponse) -> bool {
        const REFERENCE: u32 = 1 << 2;
        const TUPLE: u32 = 1 << 3;
        if ty.object_flags.unwrap_or(0) & REFERENCE != 0 && ty.target.is_some() {
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
            if target.is_some_and(|target| target.object_flags.unwrap_or(0) & TUPLE != 0) {
                return true;
            }
        }
        self.session
            .client()
            .is_tuple_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or(false)
    }

    pub(in super::super) async fn has_number_index(&mut self, ty: &TypeResponse) -> bool {
        self.session
            .client()
            .get_index_infos_of_type(
                self.session.snapshot().handle.clone(),
                self.session.project_handle(),
                ty.id.clone(),
            )
            .await
            .unwrap_or_default()
            .iter()
            .any(|info| info.key_type.flags & flags::NUMBER != 0)
    }

    pub(in super::super) async fn has_number_length(&mut self, ty: &TypeResponse) -> bool {
        let Some(length) = self.property(ty, "length").await else {
            return false;
        };
        let Some(length) = self.type_of_symbol(&length).await else {
            return false;
        };
        self.union_flags(&length).await & flags::NUMBER_LIKE != 0
    }

    /// Union/intersection structure of the base-constrained type, two levels deep.
    pub(in super::super) async fn shape(&mut self, ty: &TypeResponse) -> Option<Shape> {
        let constrained = self.base_constraint(ty).await.unwrap_or_else(|| ty.clone());
        let mut root = self.shape_of(&constrained).await;
        let mut parts = Vec::new();
        for part in self.parts(&constrained).await {
            let mut shape = self.shape_of(&part).await;
            for inner in self.parts(&part).await {
                let inner_shape = self.shape_of(&inner).await;
                shape.parts.push(inner_shape);
            }
            parts.push(shape);
        }
        root.parts = parts;
        Some(root)
    }

    pub(in super::super) async fn shape_of(&mut self, ty: &TypeResponse) -> Shape {
        let composite = ty.flags & (flags::UNION | flags::INTERSECTION) != 0;
        let object = ty.flags & flags::OBJECT != 0;
        Shape {
            flags: ty.flags,
            union: ty.flags & flags::UNION != 0,
            intersection: ty.flags & flags::INTERSECTION != 0,
            array: object && self.is_array(ty).await,
            tuple: object && self.is_tuple(ty).await,
            number_index: !composite && self.has_number_index(ty).await,
            number_length: !composite && self.has_number_length(ty).await,
            parts: Vec::new(),
        }
    }
}
