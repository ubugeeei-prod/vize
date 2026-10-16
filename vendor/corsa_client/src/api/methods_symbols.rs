//! Symbol-oriented `ApiClient` methods.
//!
//! These helpers cover name resolution and symbol lookup entry points from the
//! Corsa API. Most methods mirror upstream endpoint names closely so it is easy
//! to correlate source code, wire traces, and TypeScript checker concepts.

use serde_json::json;

use super::{
    ApiClient, DocumentIdentifier, DocumentPosition, NodeHandle, ProjectHandle, SnapshotHandle,
    SymbolHandle, SymbolResponse, TypeResponse,
    requests_core::{
        ResolveNameRequest, ShorthandValueRequest, SymbolAtLocationRequest,
        SymbolAtPositionRequest, TypeOfSymbolAtLocationRequest,
    },
    requests_symbols::{
        NodeBatchRequest, PositionBatchRequest, SymbolBatchRequest, SymbolOnlyRequest,
        SymbolProjectRequest,
    },
    requests_types::{MemberInModuleExportsRequest, TypeLocationRequest},
};
use crate::Result;

impl ApiClient {
    /// Returns the symbol visible at a specific UTF-16 position in a file.
    ///
    /// Returns `Ok(None)` when the position does not resolve to a symbol.
    pub async fn get_symbol_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        position: u32,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "getSymbolAtPosition",
            SymbolAtPositionRequest {
                snapshot,
                project,
                file: file.into(),
                position,
            },
        )
        .await
    }

    /// Resolves symbols for multiple positions in a single file.
    ///
    /// The output order matches the input `positions` order.
    pub async fn get_symbols_at_positions(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        file: impl Into<DocumentIdentifier>,
        positions: Vec<u32>,
    ) -> Result<Vec<Option<SymbolResponse>>> {
        self.call(
            "getSymbolsAtPositions",
            PositionBatchRequest {
                snapshot,
                project,
                file: file.into(),
                positions,
            },
        )
        .await
    }

    /// Returns the symbol associated with a specific syntax node.
    ///
    /// Returns `Ok(None)` when the node has no symbol binding.
    pub async fn get_symbol_at_location(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "getSymbolAtLocation",
            SymbolAtLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    /// Resolves symbols for multiple syntax nodes.
    ///
    /// The output order matches the input `locations` order.
    pub async fn get_symbols_at_locations(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        locations: Vec<NodeHandle>,
    ) -> Result<Vec<Option<SymbolResponse>>> {
        self.call(
            "getSymbolsAtLocations",
            NodeBatchRequest {
                snapshot,
                project,
                locations,
            },
        )
        .await
    }

    /// Returns the apparent checker type of a symbol.
    ///
    /// Returns `Ok(None)` when Corsa cannot associate a type with the symbol.
    pub async fn get_type_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getTypeOfSymbol",
            json!({ "snapshot": snapshot, "project": project, "symbol": symbol }),
        )
        .await
    }

    /// Returns the apparent checker types for multiple symbols.
    ///
    /// The output order matches the input `symbols` order.
    pub async fn get_types_of_symbols(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbols: Vec<SymbolHandle>,
    ) -> Result<Vec<Option<TypeResponse>>> {
        self.call(
            "getTypesOfSymbols",
            SymbolBatchRequest {
                snapshot,
                project,
                symbols,
            },
        )
        .await
    }

    /// Returns the declared type of a symbol, if any.
    ///
    /// This differs from [`Self::get_type_of_symbol`] when inference or
    /// contextual typing changes the apparent type.
    pub async fn get_declared_type_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getDeclaredTypeOfSymbol",
            json!({ "snapshot": snapshot, "project": project, "symbol": symbol }),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    /// Resolves a name through the checker using TypeScript's meaning flags.
    ///
    /// Callers can provide either a node `location` or a `(file, position)`
    /// pair, depending on which information they already have on hand.
    pub async fn resolve_name(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        name: impl Into<String>,
        meaning: u32,
        location: Option<NodeHandle>,
        file: Option<DocumentIdentifier>,
        position: Option<u32>,
        exclude_globals: Option<bool>,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "resolveName",
            ResolveNameRequest {
                snapshot,
                project,
                name: name.into(),
                location,
                file,
                position,
                meaning,
                exclude_globals,
            },
        )
        .await
    }

    /// Convenience wrapper around [`Self::resolve_name`] for file positions.
    pub async fn resolve_name_at_position(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        name: impl Into<String>,
        meaning: u32,
        location: DocumentPosition,
        exclude_globals: Option<bool>,
    ) -> Result<Option<SymbolResponse>> {
        self.resolve_name(
            snapshot,
            project,
            name,
            meaning,
            None,
            Some(location.document),
            Some(location.position),
            exclude_globals,
        )
        .await
    }

    /// Returns the value symbol referenced by a shorthand assignment node.
    ///
    /// For example, in `{ foo }`, this resolves the symbol bound to `foo`.
    pub async fn get_shorthand_assignment_value_symbol(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "getShorthandAssignmentValueSymbol",
            ShorthandValueRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    /// Returns the type of `symbol` as seen from a particular node location.
    ///
    /// This is useful when the same symbol has different contextual views at
    /// different use sites.
    pub async fn get_type_of_symbol_at_location(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
        location: NodeHandle,
    ) -> Result<Option<TypeResponse>> {
        self.call_optional(
            "getTypeOfSymbolAtLocation",
            TypeOfSymbolAtLocationRequest {
                snapshot,
                project,
                symbol,
                location,
            },
        )
        .await
    }

    /// Returns the parent symbol of `symbol`, if any.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_parent_of_symbol_in_project`] when a project
    /// handle is available.
    pub async fn get_parent_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional("getParentOfSymbol", SymbolOnlyRequest { snapshot, symbol })
            .await
    }

    /// Returns the parent symbol of `symbol`, if any.
    ///
    /// Project-scoped variant of [`ApiClient::get_parent_of_symbol`].
    pub async fn get_parent_of_symbol_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "getParentOfSymbol",
            SymbolProjectRequest {
                snapshot,
                project,
                symbol,
            },
        )
        .await
    }

    /// Returns member symbols directly attached to `symbol`.
    ///
    /// Missing server data is normalized to an empty vector.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_members_of_symbol_in_project`] when a project
    /// handle is available.
    pub async fn get_members_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        symbol: SymbolHandle,
    ) -> Result<Vec<SymbolResponse>> {
        self.call::<Option<Vec<SymbolResponse>>, _>(
            "getMembersOfSymbol",
            SymbolOnlyRequest { snapshot, symbol },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns member symbols directly attached to `symbol`.
    ///
    /// Missing server data is normalized to an empty vector.
    ///
    /// Project-scoped variant of [`ApiClient::get_members_of_symbol`].
    pub async fn get_members_of_symbol_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Vec<SymbolResponse>> {
        self.call::<Option<Vec<SymbolResponse>>, _>(
            "getMembersOfSymbol",
            SymbolProjectRequest {
                snapshot,
                project,
                symbol,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns exported symbols directly attached to `symbol`.
    ///
    /// Missing server data is normalized to an empty vector.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_exports_of_symbol_in_project`] when a project
    /// handle is available.
    pub async fn get_exports_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        symbol: SymbolHandle,
    ) -> Result<Vec<SymbolResponse>> {
        self.call::<Option<Vec<SymbolResponse>>, _>(
            "getExportsOfSymbol",
            SymbolOnlyRequest { snapshot, symbol },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns exported symbols directly attached to `symbol`.
    ///
    /// Missing server data is normalized to an empty vector.
    ///
    /// Project-scoped variant of [`ApiClient::get_exports_of_symbol`].
    pub async fn get_exports_of_symbol_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Vec<SymbolResponse>> {
        self.call::<Option<Vec<SymbolResponse>>, _>(
            "getExportsOfSymbol",
            SymbolProjectRequest {
                snapshot,
                project,
                symbol,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns the export-facing symbol associated with `symbol`.
    ///
    /// Unlike many other helpers in this group, this endpoint is expected to
    /// succeed with a concrete symbol response.
    ///
    /// TypeScript 7 stable runtimes resolve this request inside a specific
    /// project and reject project-less lookups; prefer
    /// [`ApiClient::get_export_symbol_of_symbol_in_project`] when a project
    /// handle is available.
    pub async fn get_export_symbol_of_symbol(
        &self,
        snapshot: SnapshotHandle,
        symbol: SymbolHandle,
    ) -> Result<SymbolResponse> {
        self.call(
            "getExportSymbolOfSymbol",
            SymbolOnlyRequest { snapshot, symbol },
        )
        .await
    }

    /// Returns the export-facing symbol associated with `symbol`.
    ///
    /// Unlike many other helpers in this group, this endpoint is expected to
    /// succeed with a concrete symbol response.
    ///
    /// Project-scoped variant of [`ApiClient::get_export_symbol_of_symbol`].
    pub async fn get_export_symbol_of_symbol_in_project(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<SymbolResponse> {
        self.call(
            "getExportSymbolOfSymbol",
            SymbolProjectRequest {
                snapshot,
                project,
                symbol,
            },
        )
        .await
    }

    /// Returns the symbol an alias resolves to, following the whole chain.
    pub async fn get_aliased_symbol(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_symbol_property("getAliasedSymbol", snapshot, project, symbol)
            .await
    }

    /// Returns the symbol an alias resolves to, following one step only.
    pub async fn get_immediate_aliased_symbol(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_symbol_property("getImmediateAliasedSymbol", snapshot, project, symbol)
            .await
    }

    /// Returns the exported symbols of a module symbol.
    ///
    /// Missing server data is normalized to an empty vector.
    pub async fn get_exports_of_module(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Vec<SymbolResponse>> {
        self.call::<Option<Vec<SymbolResponse>>, _>(
            "getExportsOfModule",
            SymbolProjectRequest {
                snapshot,
                project,
                symbol,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns a named export of a module symbol, if it has one.
    pub async fn get_member_in_module_exports(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
        name: impl Into<String>,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "getMemberInModuleExports",
            MemberInModuleExportsRequest {
                snapshot,
                project,
                symbol,
                name: name.into(),
            },
        )
        .await
    }

    /// Returns the local target symbol of an export specifier node.
    pub async fn get_export_specifier_local_target_symbol(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            "getExportSpecifierLocalTargetSymbol",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    /// Returns the JSDoc tags attached to a symbol.
    ///
    /// Missing server data is normalized to an empty vector.
    pub async fn get_js_doc_tags(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Vec<super::JsDocTagInfo>> {
        self.call::<Option<Vec<super::JsDocTagInfo>>, _>(
            "getJsDocTags",
            SymbolProjectRequest {
                snapshot,
                project,
                symbol,
            },
        )
        .await
        .map(|items| items.unwrap_or_default())
    }

    /// Returns the rendered documentation comment of a symbol.
    pub async fn get_documentation_comment(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<String> {
        self.call(
            "getDocumentationComment",
            SymbolProjectRequest {
                snapshot,
                project,
                symbol,
            },
        )
        .await
    }

    /// Returns the compile-time constant an enum member or literal node folds
    /// to, if the checker can compute one.
    pub async fn get_constant_value(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        location: NodeHandle,
    ) -> Result<Option<serde_json::Value>> {
        self.call_optional(
            "getConstantValue",
            TypeLocationRequest {
                snapshot,
                project,
                location,
            },
        )
        .await
    }

    /// Returns handles for the checker's intrinsic well-known symbols.
    pub async fn get_well_known_symbols(
        &self,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
    ) -> Result<super::WellKnownSymbolsResponse> {
        self.call(
            "getWellKnownSymbols",
            super::requests_core::IntrinsicTypeRequest { snapshot, project },
        )
        .await
    }

    /// Shared helper for endpoints that map one symbol to another.
    async fn call_symbol_property(
        &self,
        method: &str,
        snapshot: SnapshotHandle,
        project: ProjectHandle,
        symbol: SymbolHandle,
    ) -> Result<Option<SymbolResponse>> {
        self.call_optional(
            method,
            SymbolProjectRequest {
                snapshot,
                project,
                symbol,
            },
        )
        .await
    }
}
