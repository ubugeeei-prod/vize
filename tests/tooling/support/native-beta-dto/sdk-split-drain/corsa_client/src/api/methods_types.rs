//! Type-oriented `ApiClient` methods.
//!
//! These helpers mostly expose TypeScript checker queries. They are useful for
//! "what type is this?" style workflows, intrinsic type access, and for turning
//! Corsa's opaque type handles back into richer structural information.

use base64::{Engine as _, engine::general_purpose::STANDARD};

use super::{
    ApiClient, DocumentIdentifier, EncodedPayload, NodeHandle, ProjectHandle, SnapshotHandle,
    TypeHandle, TypeResponse,
    encoded::PrintNodeOptions,
    requests_core::{IntrinsicTypeRequest, TypeNodeRequest},
    requests_symbols::{PositionBatchRequest, TypeProjectRequest},
    requests_types::{
        PrintNodeRequest, PropertyOfTypeRequest, SignatureOfTypeRequest, TypeAssignabilityRequest,
        TypeLocationRequest,
    },
};
use crate::Result;

impl ApiClient {
    /// Returns the checker type associated with a syntax node.
    ///
    /// Returns `Ok(None)` when the location has no associated type.
    pub async fn get_type_at_location(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getTypeAtLocation",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    /// Resolves types for multiple syntax nodes.
    ///
    /// The output order matches the input `locations` order.
    pub async fn get_type_at_locations(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        locations: Vec<NodeHandle>,
    ) -> Result<Vec<Option<TypeResponse>>> {
        self.call(
            "getTypeAtLocations",
            super::requests_symbols::NodeBatchRequest {
                snapshot,
                project,
                locations,
            },
        )
        .await
    }

    /// Returns the checker type visible at a UTF-16 position in a file.
    ///
    /// Returns `Ok(None)` when the position does not correspond to a typed
    /// entity.
    pub async fn get_type_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getTypeAtPosition",
            super::requests_core::SymbolAtPositionRequest {
                snapshot,
                project,
                file: file.into(),
                position,
            },
        )
        .await
    }

    /// Resolves types for multiple positions in a single file.
    ///
    /// The output order matches the input `positions` order.
    pub async fn get_types_at_positions(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        positions: Vec<u32>,
    ) -> Result<Vec<Option<TypeResponse>>> {
        self.call(
            "getTypesAtPositions",
            PositionBatchRequest {
                snapshot,
                project,
                file: file.into(),
                positions,
            },
        )
        .await
    }

    /// Returns signatures of a type for the given signature `kind`.
    ///
    /// `kind` is forwarded directly to the upstream checker endpoint.
    pub async fn get_signatures_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
        kind: i32,
    ) -> Result<Vec<super::SignatureResponse>> {
        self.call(
            "getSignaturesOfType",
            SignatureOfTypeRequest {
                snapshot,
                project,
                r#type,
                kind,
            },
        )
        .await
    }

    /// Returns the contextual type associated with a syntax node.
    ///
    /// This is especially useful for function expressions and object literals
    /// whose types are influenced by surrounding context.
    pub async fn get_contextual_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getContextualType",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    /// Returns the widened base type of a literal type.
    pub async fn get_base_type_of_literal_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getBaseTypeOfLiteralType",
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns whether `source` is assignable to `target`.
    ///
    /// This is the checker's own assignability relation, and the primitive most
    /// type-aware lint rules are built on. Prefer it over comparing rendered
    /// type texts, which cannot model structural assignability.
    pub async fn is_type_assignable_to(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        source: TypeHandle,
        target: TypeHandle,
    ) -> Result<bool> {
        self.call(
            "isTypeAssignableTo",
            TypeAssignabilityRequest {
                snapshot,
                project,
                source,
                target,
            },
        )
        .await
    }

    /// Returns whether a type is an array type.
    pub async fn is_array_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<bool> {
        self.call_type_predicate("isArrayType", snapshot, project, r#type)
            .await
    }

    /// Returns whether a type is a tuple type.
    pub async fn is_tuple_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<bool> {
        self.call_type_predicate("isTupleType", snapshot, project, r#type)
            .await
    }

    /// Returns whether a type is array-like.
    pub async fn is_array_like_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<bool> {
        self.call_type_predicate("isArrayLikeType", snapshot, project, r#type)
            .await
    }

    /// Returns the apparent type of a type.
    ///
    /// This resolves type parameters to their constraints and primitives to
    /// their wrapper types, which is what property lookups see.
    pub async fn get_apparent_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_type_property("getApparentType", snapshot, project, r#type)
            .await
    }

    /// Returns the type with `null` and `undefined` removed.
    pub async fn get_non_nullable_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_type_property("getNonNullableType", snapshot, project, r#type)
            .await
    }

    /// Returns the widened form of a type, turning inference literals into
    /// their base types.
    pub async fn get_widened_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_type_property("getWidenedType", snapshot, project, r#type)
            .await
    }

    /// Returns the base constraint of a type parameter or conditional type.
    pub async fn get_base_constraint_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_type_property("getBaseConstraintOfType", snapshot, project, r#type)
            .await
    }

    /// Returns the fresh form of a literal type.
    pub async fn get_fresh_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_type_property("getFreshTypeOfType", snapshot, project, r#type)
            .await
    }

    /// Returns the regular (non-fresh) form of a literal type.
    pub async fn get_regular_type_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_type_property("getRegularTypeOfType", snapshot, project, r#type)
            .await
    }

    /// Returns the `true` branch type of a conditional type.
    pub async fn get_true_type_of_conditional_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_type_property("getTrueTypeOfConditionalType", snapshot, project, r#type)
            .await
    }

    /// Returns the `false` branch type of a conditional type.
    pub async fn get_false_type_of_conditional_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_type_property("getFalseTypeOfConditionalType", snapshot, project, r#type)
            .await
    }

    /// Returns a named property symbol of a type, if it has one.
    pub async fn get_property_of_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
        name: impl Into<String>,
    ) -> Result<Option<super::SymbolResponse>> {
        self.call_optional(
            "getPropertyOfType",
            PropertyOfTypeRequest {
                snapshot,
                project,
                r#type,
                name: name.into(),
            },
        )
        .await
    }

    /// Returns the type a type node denotes.
    pub async fn get_type_from_type_node(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getTypeFromTypeNode",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    /// Shared helper for boolean type predicates.
    async fn call_type_predicate(
        &self,
        method: &str,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<bool> {
        self.call(
            method,
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Shared helper for endpoints that map one type to another.
    async fn call_type_property(
        &self,
        method: &str,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            method,
            TypeProjectRequest {
                snapshot,
                project,
                r#type,
            },
        )
        .await
    }

    /// Returns the intrinsic `any` type for the project.
    pub async fn get_any_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getAnyType", snapshot, project).await
    }

    /// Returns the intrinsic `string` type for the project.
    pub async fn get_string_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getStringType", snapshot, project)
            .await
    }

    /// Returns the intrinsic `number` type for the project.
    pub async fn get_number_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getNumberType", snapshot, project)
            .await
    }

    /// Returns the intrinsic `boolean` type for the project.
    pub async fn get_boolean_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getBooleanType", snapshot, project)
            .await
    }

    /// Returns the intrinsic `void` type for the project.
    pub async fn get_void_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getVoidType", snapshot, project).await
    }

    /// Returns the intrinsic `undefined` type for the project.
    pub async fn get_undefined_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getUndefinedType", snapshot, project)
            .await
    }

    /// Returns the intrinsic `null` type for the project.
    pub async fn get_null_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getNullType", snapshot, project).await
    }

    /// Returns the intrinsic `never` type for the project.
    pub async fn get_never_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getNeverType", snapshot, project).await
    }

    /// Returns the intrinsic `unknown` type for the project.
    pub async fn get_unknown_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getUnknownType", snapshot, project)
            .await
    }

    /// Returns the intrinsic `bigint` type for the project.
    pub async fn get_big_int_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getBigIntType", snapshot, project)
            .await
    }

    /// Returns the intrinsic ECMAScript `symbol` type for the project.
    pub async fn get_es_symbol_type(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call_intrinsic("getESSymbolType", snapshot, project)
            .await
    }

    /// Converts a type into a serialized type-node payload.
    ///
    /// The returned payload can be fed into [`Self::print_node`] for text
    /// rendering, or into other node-oriented helpers that understand the
    /// binary node encoding.
    pub async fn type_to_type_node(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
        location: Option<NodeHandle>,
        flags: Option<i32>,
    ) -> Result<Option<EncodedPayload>> {
        self.call_optional_binary(
            "typeToTypeNode",
            TypeNodeRequest {
                snapshot,
                project,
                r#type,
                location,
                flags,
            },
        )
        .await
    }

    /// Renders a type to text using Corsa's checker printer.
    pub async fn type_to_string(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        r#type: TypeHandle,
        location: Option<NodeHandle>,
        flags: Option<i32>,
    ) -> Result<String> {
        self.call(
            "typeToString",
            TypeNodeRequest {
                snapshot,
                project,
                r#type,
                location,
                flags,
            },
        )
        .await
    }

    /// Returns whether a node is treated as context sensitive by the checker.
    pub async fn is_context_sensitive(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<bool> {
        self.call(
            "isContextSensitive",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    /// Renders a serialized node payload into source text.
    ///
    /// `payload` is expected to come from binary endpoints such as
    /// [`Self::type_to_type_node`] or [`Self::get_source_file`].
    pub async fn print_node(
        &self,
        payload: &EncodedPayload,
        options: PrintNodeOptions,
    ) -> Result<String> {
        if !self.allows_unstable_upstream_calls() {
            return Err(crate::CorsaError::Unsupported(
                "printNode is disabled by default because upstream can panic on real project data; opt in with ApiSpawnConfig::with_allow_unstable_upstream_calls(true)",
            ));
        }
        self.call(
            "printNode",
            PrintNodeRequest {
                data: STANDARD.encode(payload.as_bytes()),
                preserve_source_newlines: options.preserve_source_newlines,
                never_ascii_escape: options.never_ascii_escape,
                terminate_unterminated_literals: options.terminate_unterminated_literals,
            },
        )
        .await
    }

    /// Shared helper for intrinsic type endpoints.
    async fn call_intrinsic(
        &self,
        method: &str,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<TypeResponse> {
        self.call(method, IntrinsicTypeRequest { snapshot, project })
            .await
    }
}
