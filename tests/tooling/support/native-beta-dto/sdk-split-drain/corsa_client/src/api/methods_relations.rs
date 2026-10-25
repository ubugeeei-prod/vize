//! Relationship-oriented `ApiClient` methods.
//!
//! This group answers questions about how types, signatures, and symbols relate
//! to each other after the checker has already identified them.

use super::{
    ApiClient, IndexInfo, ProjectHandle, SignatureHandle, SnapshotHandle, SymbolHandle, TypeHandle,
    TypePredicateResponse, TypeResponse,
    requests_symbols::{SignatureOnlyRequest, TypeOnlyRequest, TypeProjectRequest},
    requests_types::{ParameterTypeRequest, TypeLocationRequest},
};
use crate::Result;
use corsa_core::utils::split_top_level_type_text;

impl ApiClient {
    /// Returns the symbol attached to a type, if one exists.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_symbol_of_type_in_project`] when a project handle is
    /// available.
    pub async fn get_symbol_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<super::SymbolResponse>> {
        self.call_optional("getSymbolOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    /// Returns the symbol attached to a type, resolved inside a project.
    ///
    /// This is the project-scoped variant of [`ApiClient::get_symbol_of_type`]
    /// that TypeScript 7 stable runtimes require.
    pub async fn get_symbol_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<super::SymbolResponse>> {
        self.call_optional(
            "getSymbolOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns the return type of a signature.
    pub async fn get_return_type_of_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getReturnTypeOfSignature",
            SignatureOnlyRequest {
                snapshot,
                project,
                signature,
            },
        )
        .await
    }

    /// Returns the signature the checker resolves a call-like node to.
    ///
    /// This is the entry point for rules that reason about an actual call
    /// rather than a type's declared signatures.
    pub async fn get_resolved_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: super::NodeHandle,
    ) -> Result<Option<super::SignatureResponse>> {
        self.call_optional(
            "getResolvedSignature",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    /// Returns the signature a declaration node produces.
    pub async fn get_signature_from_declaration(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: super::NodeHandle,
    ) -> Result<Option<super::SignatureResponse>> {
        self.call_optional(
            "getSignatureFromDeclaration",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    /// Returns the generic type parameters declared on a signature.
    ///
    /// Missing server data is normalized to an empty vector.
    pub async fn get_type_parameters_of_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getTypeParametersOfSignature",
            SignatureOnlyRequest {
                snapshot,
                project,
                signature,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns the target signature an instantiated signature came from.
    pub async fn get_target_of_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Option<super::SignatureResponse>> {
        self.call_optional(
            "getTargetOfSignature",
            SignatureOnlyRequest {
                snapshot,
                project,
                signature,
            },
        )
        .await
    }

    /// Returns the type of the parameter at `index` in a signature.
    ///
    /// Unlike resolving the parameter symbol and asking for its type, this
    /// accounts for rest parameters expanded at the given position.
    pub async fn get_parameter_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
        index: i32,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getParameterType",
            ParameterTypeRequest {
                snapshot,
                project,
                signature,
                index,
            },
        )
        .await
    }

    /// Returns the alias symbol a type was written through, if any.
    pub async fn get_alias_symbol_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<super::SymbolResponse>> {
        self.call_optional(
            "getAliasSymbolOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns the type arguments supplied to a type's alias, if any.
    ///
    /// Missing server data is normalized to an empty vector.
    pub async fn get_alias_type_arguments_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getAliasTypeArgumentsOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns the parameter symbols of a signature, in declaration order.
    ///
    /// This is the checker's own answer, including each parameter's name and
    /// declarations. Missing server data is normalized to an empty vector.
    pub async fn get_parameters_of_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Vec<super::SymbolResponse>> {
        self.call::<Option<Vec<super::SymbolResponse>>, _>(
            "getParametersOfSignature",
            SignatureOnlyRequest {
                snapshot,
                project,
                signature,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns the explicit `this` parameter symbol of a signature, if any.
    pub async fn get_this_parameter_of_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Option<super::SymbolResponse>> {
        self.call_optional(
            "getThisParameterOfSignature",
            SignatureOnlyRequest {
                snapshot,
                project,
                signature,
            },
        )
        .await
    }

    /// Returns the rest type of a signature, if any.
    pub async fn get_rest_type_of_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getRestTypeOfSignature",
            SignatureOnlyRequest {
                snapshot,
                project,
                signature,
            },
        )
        .await
    }

    /// Returns the type predicate declared on a signature, if any.
    pub async fn get_type_predicate_of_signature(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Option<TypePredicateResponse>> {
        self.call_optional(
            "getTypePredicateOfSignature",
            SignatureOnlyRequest {
                snapshot,
                project,
                signature,
            },
        )
        .await
    }

    /// Returns the immediate base types of a type.
    ///
    /// Missing server data is normalized to an empty vector.
    pub async fn get_base_types(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.get_base_types_with_texts(snapshot, project, r#type, &[])
            .await
    }

    /// Returns the immediate base types of a type, using already-rendered type
    /// text as a fallback when the upstream handle can no longer be rendered.
    pub async fn get_base_types_with_texts(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
        type_texts: &[String],
    ) -> Result<Vec<TypeResponse>> {
        let bases = self
            .get_base_types_direct(snapshot.clone(), project.clone(), r#type.clone())
            .await?;
        if !bases.is_empty() {
            return Ok(bases);
        }

        if let Some(target) = self
            .get_target_of_type_or_none(snapshot.clone(), project.clone(), r#type.clone())
            .await?
            .filter(|target| target.id != r#type)
        {
            let target_bases = self
                .get_base_types_direct(snapshot.clone(), project.clone(), target.id)
                .await?;
            if !target_bases.is_empty() {
                return Ok(target_bases);
            }
        }

        let mut construct_return_bases = Vec::new();
        let construct_signatures = self
            .get_signatures_of_type_or_empty(snapshot.clone(), project.clone(), r#type.clone(), 1)
            .await?;
        for signature in construct_signatures {
            let Some(return_type) = self
                .get_return_type_of_signature_or_none(
                    snapshot.clone(),
                    project.clone(),
                    signature.id,
                )
                .await?
                .filter(|return_type| return_type.id != r#type)
            else {
                continue;
            };
            let mut return_bases = self
                .get_base_types_direct(snapshot.clone(), project.clone(), return_type.id.clone())
                .await?;
            if return_bases.is_empty() {
                if let Some(target) = self
                    .get_target_of_type_or_none(
                        snapshot.clone(),
                        project.clone(),
                        return_type.id.clone(),
                    )
                    .await?
                    .filter(|target| target.id != return_type.id)
                {
                    return_bases = self
                        .get_base_types_direct(snapshot.clone(), project.clone(), target.id)
                        .await?;
                }
            }
            if return_bases.is_empty() {
                return_bases = self
                    .get_types_of_type_in_project(snapshot.clone(), project.clone(), return_type.id)
                    .await?;
            }
            append_unique_types(&mut construct_return_bases, return_bases);
        }
        if !construct_return_bases.is_empty() {
            return Ok(construct_return_bases);
        }

        let intersection_parts = self
            .get_types_of_type_in_project(snapshot.clone(), project.clone(), r#type.clone())
            .await?;
        if !intersection_parts.is_empty() {
            return Ok(intersection_parts);
        }

        let text_parts = self
            .fallback_intersection_parts_from_text(snapshot, project, r#type.clone())
            .await?;
        if !text_parts.is_empty() {
            return Ok(text_parts);
        }

        let hinted_text_parts =
            fallback_intersection_parts_from_type_texts(r#type.as_str(), type_texts);
        if !hinted_text_parts.is_empty() {
            return Ok(hinted_text_parts);
        }

        Ok(Vec::new())
    }

    async fn get_base_types_direct(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        match self
            .call::<Option<Vec<TypeResponse>>, _>(
                "getBaseTypes",
                TypeProjectRequest {
                    snapshot,
                    project,
                    r#type,
                },
            )
            .await
        {
            Ok(items) => Ok(items.unwrap_or_default()),
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                Ok(Vec::new())
            }
            Err(error) => Err(error),
        }
    }

    async fn get_target_of_type_or_none(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        match self
            .get_target_of_type_in_project(snapshot, project, r#type)
            .await
        {
            Ok(target) => Ok(target),
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    async fn get_signatures_of_type_or_empty(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
        kind: i32,
    ) -> Result<Vec<super::SignatureResponse>> {
        match self
            .get_signatures_of_type(snapshot, project, r#type, kind)
            .await
        {
            Ok(signatures) => Ok(signatures),
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                Ok(Vec::new())
            }
            Err(error) => Err(error),
        }
    }

    async fn get_return_type_of_signature_or_none(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        signature: SignatureHandle,
    ) -> Result<Option<TypeResponse>> {
        match self
            .get_return_type_of_signature(snapshot, project, signature)
            .await
        {
            Ok(return_type) => Ok(return_type),
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    /// Returns the properties exposed by a type.
    ///
    /// Missing server data is normalized to an empty vector.
    pub async fn get_properties_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<super::SymbolResponse>> {
        self.call::<Option<Vec<super::SymbolResponse>>, _>(
            "getPropertiesOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns index signature information for a type.
    ///
    /// Missing server data is normalized to an empty vector.
    pub async fn get_index_infos_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<IndexInfo>> {
        self.call::<Option<Vec<IndexInfo>>, _>(
            "getIndexInfosOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns the constraint of a type parameter, if one exists.
    pub async fn get_constraint_of_type_parameter(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getConstraintOfTypeParameter",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns the type arguments of an instantiated type.
    ///
    /// Missing server data is normalized to an empty vector. A stale type
    /// handle that the server has dropped from its snapshot registry is also
    /// treated as "no type arguments" so callers can keep analyzing the type.
    pub async fn get_type_arguments(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        match self
            .call::<Option<Vec<TypeResponse>>, _>(
                "getTypeArguments",
                TypeProjectRequest {
                    snapshot: snapshot.clone(),
                    project: project.clone(),
                    r#type: r#type.clone(),
                },
            )
            .await
        {
            Ok(Some(items)) if !items.is_empty() => {
                self.structural_type_arguments_from_symbols(snapshot, project, items)
                    .await
            }
            Ok(_) => {
                self.fallback_type_arguments_from_text(snapshot, project, r#type)
                    .await
            }
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                self.fallback_type_arguments_from_text(snapshot, project, r#type)
                    .await
            }
            Err(error) => Err(error),
        }
    }

    async fn structural_type_arguments_from_symbols(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        items: Vec<TypeResponse>,
    ) -> Result<Vec<TypeResponse>> {
        let mut structural = Vec::with_capacity(items.len());
        for item in items {
            let Some(symbol) = item.symbol.clone() else {
                structural.push(item);
                continue;
            };
            let Some(candidate) = self
                .symbol_type_or_none(snapshot.clone(), project.clone(), symbol)
                .await?
            else {
                structural.push(item);
                continue;
            };
            if self
                .same_type_text(snapshot.clone(), project.clone(), &item, &candidate)
                .await?
            {
                structural.push(candidate);
            } else {
                structural.push(item);
            }
        }
        Ok(structural)
    }

    async fn symbol_type_or_none(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<TypeResponse>> {
        match self
            .get_declared_type_of_symbol(snapshot.clone(), project.clone(), symbol.clone())
            .await
        {
            Ok(Some(r#type)) => Ok(Some(r#type)),
            Ok(None) => self.type_of_symbol_or_none(snapshot, project, symbol).await,
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    async fn type_of_symbol_or_none(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<TypeResponse>> {
        match self.get_type_of_symbol(snapshot, project, symbol).await {
            Ok(r#type) => Ok(r#type),
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    async fn same_type_text(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        left: &TypeResponse,
        right: &TypeResponse,
    ) -> Result<bool> {
        if left.id == right.id || type_response_texts_overlap(left, right) {
            return Ok(true);
        }
        let Some(left_text) = self
            .type_response_text_or_none(snapshot.clone(), project.clone(), left)
            .await?
        else {
            return Ok(false);
        };
        let Some(right_text) = self
            .type_response_text_or_none(snapshot, project, right)
            .await?
        else {
            return Ok(false);
        };
        Ok(left_text == right_text)
    }

    async fn type_response_text_or_none(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: &TypeResponse,
    ) -> Result<Option<String>> {
        if let Some(text) = r#type.texts.iter().find(|text| !text.is_empty()) {
            return Ok(Some(text.clone()));
        }
        match self
            .type_to_string(snapshot, project, r#type.id.clone(), None, None)
            .await
        {
            Ok(text) => Ok(Some(text)),
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    /// Returns the target type underlying an instantiated or mapped type.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project; prefer [`ApiClient::get_target_of_type_in_project`] when a
    /// project handle is available.
    pub async fn get_target_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getTargetOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    /// Returns the target type underlying an instantiated or mapped type,
    /// resolved inside a project.
    pub async fn get_target_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getTargetOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns nested or constituent types associated with a type.
    ///
    /// Missing server data is normalized to an empty vector. TypeScript 7
    /// stable runtimes resolve this request inside a specific project; prefer
    /// [`ApiClient::get_types_of_type_in_project`] when a project handle is
    /// available.
    pub async fn get_types_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        match self
            .call::<Option<Vec<TypeResponse>>, _>(
                "getTypesOfType",
                TypeOnlyRequest { snapshot, r#type },
            )
            .await
        {
            Ok(items) => Ok(items.unwrap_or_default()),
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                Ok(Vec::new())
            }
            Err(error) => Err(error),
        }
    }

    /// Returns nested or constituent types associated with a type, resolved
    /// inside a project.
    ///
    /// Missing server data is normalized to an empty vector.
    pub async fn get_types_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        match self
            .call::<Option<Vec<TypeResponse>>, _>(
                "getTypesOfType",
                TypeProjectRequest {
                    snapshot,
                    project,
                    r#type,
                },
            )
            .await
        {
            Ok(items) => Ok(items.unwrap_or_default()),
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                Ok(Vec::new())
            }
            Err(error) => Err(error),
        }
    }

    /// Returns the direct type parameters declared on a type.
    ///
    /// Missing server data is normalized to an empty vector.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_type_parameters_of_type_in_project`] when a project
    /// handle is available.
    pub async fn get_type_parameters_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getTypeParametersOfType",
            TypeOnlyRequest { snapshot, r#type },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns the direct type parameters declared on a type.
    ///
    /// Project-scoped variant of [`ApiClient::get_type_parameters_of_type`].
    /// Missing server data is normalized to an empty vector.
    pub async fn get_type_parameters_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getTypeParametersOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns outer type parameters captured by a type.
    ///
    /// Missing server data is normalized to an empty vector.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_outer_type_parameters_of_type_in_project`] when a project
    /// handle is available.
    pub async fn get_outer_type_parameters_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getOuterTypeParametersOfType",
            TypeOnlyRequest { snapshot, r#type },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns outer type parameters captured by a type.
    ///
    /// Project-scoped variant of [`ApiClient::get_outer_type_parameters_of_type`].
    /// Missing server data is normalized to an empty vector.
    pub async fn get_outer_type_parameters_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getOuterTypeParametersOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns local type parameters introduced while resolving a type.
    ///
    /// Missing server data is normalized to an empty vector.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_local_type_parameters_of_type_in_project`] when a project
    /// handle is available.
    pub async fn get_local_type_parameters_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getLocalTypeParametersOfType",
            TypeOnlyRequest { snapshot, r#type },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns local type parameters introduced while resolving a type.
    ///
    /// Project-scoped variant of [`ApiClient::get_local_type_parameters_of_type`].
    /// Missing server data is normalized to an empty vector.
    pub async fn get_local_type_parameters_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        self.call::<Option<Vec<TypeResponse>>, _>(
            "getLocalTypeParametersOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns the object side of a wrapper type, if one exists.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_object_type_of_type_in_project`] when a project
    /// handle is available.
    pub async fn get_object_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getObjectTypeOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    /// Returns the object side of a wrapper type, if one exists.
    ///
    /// Project-scoped variant of [`ApiClient::get_object_type_of_type`].
    pub async fn get_object_type_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getObjectTypeOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns the index side of a wrapper type, if one exists.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_index_type_of_type_in_project`] when a project
    /// handle is available.
    pub async fn get_index_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getIndexTypeOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    /// Returns the index side of a wrapper type, if one exists.
    ///
    /// Project-scoped variant of [`ApiClient::get_index_type_of_type`].
    pub async fn get_index_type_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getIndexTypeOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns the check side of a wrapper or conditional type, if one exists.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_check_type_of_type_in_project`] when a project
    /// handle is available.
    pub async fn get_check_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getCheckTypeOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    /// Returns the check side of a wrapper or conditional type, if one exists.
    ///
    /// Project-scoped variant of [`ApiClient::get_check_type_of_type`].
    pub async fn get_check_type_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getCheckTypeOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns the `extends` side of a conditional type, if one exists.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_extends_type_of_type_in_project`] when a project
    /// handle is available.
    pub async fn get_extends_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getExtendsTypeOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    /// Returns the `extends` side of a conditional type, if one exists.
    ///
    /// Project-scoped variant of [`ApiClient::get_extends_type_of_type`].
    pub async fn get_extends_type_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getExtendsTypeOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns the base type recorded directly on a type, if one exists.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_base_type_of_type_in_project`] when a project
    /// handle is available.
    pub async fn get_base_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional("getBaseTypeOfType", TypeOnlyRequest { snapshot, r#type })
            .await
    }

    /// Returns the base type recorded directly on a type, if one exists.
    ///
    /// Project-scoped variant of [`ApiClient::get_base_type_of_type`].
    pub async fn get_base_type_of_type_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getBaseTypeOfType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns the constraint recorded directly on a type, if one exists.
    pub async fn get_constraint_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        if let Some(constraint) = self
            .get_constraint_of_type_parameter(snapshot.clone(), project.clone(), r#type.clone())
            .await?
        {
            return Ok(Some(constraint));
        }
        self.get_constraint_of_type_direct(snapshot, project, r#type)
            .await
    }

    async fn get_constraint_of_type_direct(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        match self
            .call_optional(
                "getConstraintOfType",
                TypeProjectRequest {
                    snapshot,
                    project,
                    r#type,
                },
            )
            .await
        {
            Ok(constraint) => Ok(constraint),
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    async fn fallback_type_arguments_from_text(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        let text = match self
            .type_to_string(snapshot, project, r#type.clone(), None, None)
            .await
        {
            Ok(text) => text,
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                return Ok(Vec::new());
            }
            Err(error) => return Err(error),
        };
        let Some(arguments) = generic_argument_text(&text) else {
            return Ok(Vec::new());
        };
        Ok(split_top_level_type_text(arguments, ',')
            .into_iter()
            .enumerate()
            .map(|(index, text)| synthetic_type_response(r#type.as_str(), index, text))
            .collect())
    }

    async fn fallback_intersection_parts_from_text(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Vec<TypeResponse>> {
        let text = match self
            .type_to_string(snapshot, project, r#type.clone(), None, None)
            .await
        {
            Ok(text) => text,
            Err(error)
                if Self::is_stale_handle_error(&error) || Self::is_protocol_panic_error(&error) =>
            {
                return Ok(Vec::new());
            }
            Err(error) => return Err(error),
        };
        let parts = split_top_level_type_text(text.as_str(), '&');
        if parts.len() <= 1 {
            return Ok(Vec::new());
        }
        Ok(parts
            .into_iter()
            .enumerate()
            .map(|(index, text)| synthetic_type_response(r#type.as_str(), index, text))
            .collect())
    }
}

fn append_unique_types(target: &mut Vec<TypeResponse>, source: Vec<TypeResponse>) {
    for item in source {
        if target.iter().any(|existing| existing.id == item.id) {
            continue;
        }
        target.push(item);
    }
}

fn type_response_texts_overlap(left: &TypeResponse, right: &TypeResponse) -> bool {
    left.texts
        .iter()
        .filter(|text| !text.is_empty())
        .any(|left_text| right.texts.iter().any(|right_text| left_text == right_text))
}

fn fallback_intersection_parts_from_type_texts(
    source_type: &str,
    type_texts: &[String],
) -> Vec<TypeResponse> {
    for text in type_texts {
        let parts = split_top_level_type_text(text.as_str(), '&');
        if parts.len() <= 1 {
            continue;
        }
        return parts
            .into_iter()
            .enumerate()
            .map(|(index, text)| synthetic_type_response(source_type, index, text))
            .collect();
    }
    Vec::new()
}

fn generic_argument_text(text: &str) -> Option<&str> {
    let text = text.trim();
    let mut angle_depth = 0usize;
    let mut square_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut quote = None;
    let mut open = None;
    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            '<' if angle_depth == 0
                && square_depth == 0
                && paren_depth == 0
                && brace_depth == 0 =>
            {
                open.get_or_insert(index);
                angle_depth += 1;
            }
            '<' => angle_depth += 1,
            '>' if angle_depth > 0 => angle_depth -= 1,
            '[' => square_depth += 1,
            ']' if square_depth > 0 => square_depth -= 1,
            '(' => paren_depth += 1,
            ')' if paren_depth > 0 => paren_depth -= 1,
            '{' => brace_depth += 1,
            '}' if brace_depth > 0 => brace_depth -= 1,
            _ => {}
        }
    }
    let open = open?;
    if angle_depth != 0 || !text.ends_with('>') {
        return None;
    }
    Some(&text[open + 1..text.len() - 1])
}

fn synthetic_type_response(source_type: &str, index: usize, text: String) -> TypeResponse {
    TypeResponse {
        id: TypeHandle::from(format!(
            "synthetic-type-argument:{source_type}:{index}:{text}"
        )),
        flags: 0,
        object_flags: None,
        value: None,
        target: None,
        type_parameters: Vec::new(),
        outer_type_parameters: Vec::new(),
        local_type_parameters: Vec::new(),
        element_flags: Vec::new(),
        fixed_length: None,
        readonly: None,
        object_type: None,
        index_type: None,
        check_type: None,
        extends_type: None,
        base_type: None,
        subst_constraint: None,
        texts: vec![text],
        symbol: None,
    }
}
