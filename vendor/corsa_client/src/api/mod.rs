//! Typed request/response layer for the Corsa stdio API.
//!
//! The module is organized around a few concepts:
//!
//! - configuration and spawn types such as [`ApiSpawnConfig`] and [`ApiProfile`]
//! - lifecycle primitives such as [`ApiClient`] and [`ManagedSnapshot`]
//! - strongly typed document, handle, and response models used by endpoint
//!   helpers
//! - transport-specific helpers kept internal to the module tree
//!
//! Most consumers should start with [`ApiClient::spawn`](crate::ApiClient::spawn)
//! and [`ApiClient::update_snapshot`](crate::ApiClient::update_snapshot).

mod callbacks;
mod capabilities;
mod changes;
mod client;
mod config;
mod content_mapper;
mod diagnostics;
mod document;
mod driver;
mod encoded;
mod handles;
mod methods_diagnostics;
mod methods_editor;
mod methods_relations;
mod methods_symbols;
mod methods_types;
mod msgpack_codec;
mod msgpack_worker;
mod profiling;
mod project_session;
mod project_session_capabilities;
mod project_session_diagnostics;
mod project_session_editor;
mod project_session_probe;
mod requests_core;
mod requests_editor;
mod requests_symbols;
mod requests_types;
mod responses;
mod semantics;
mod snapshot;
mod source_file;
mod spawn_stdio;
mod type_probe;

/// Filesystem callback traits and helper functions used by spawned workers.
pub use callbacks::{
    ApiFileSystem, DirectoryEntries, FileSystemCapabilities, ReadFileResult, callback_flag,
    callback_names,
};
/// Runtime capability descriptors exposed by `describeCapabilities`.
pub use capabilities::{
    CapabilitiesResponse, DiagnosticsCapabilities, EditorCapabilities, LspCapabilities,
    OverlayCapabilities, RuntimeCapabilities,
};
/// Snapshot update inputs and change summaries.
pub use changes::{
    FileChangeSummary, FileChanges, OverlayChanges, OverlayUpdate, SnapshotChanges,
    UpdateSnapshotParams,
};
/// High-level Corsa API client.
pub use client::ApiClient;
/// Spawn-time transport and profile configuration.
pub use config::{ApiMode, ApiProfile, ApiSpawnConfig};
/// Content mapper definitions and virtual/original position mapping.
pub use content_mapper::{
    ContentMapperDefinition, DiagnosticDirectivePolicy, MappedDiagnosticDirective, MappedPosition,
    MappedRange, SUPPORTED_VIRTUAL_EXTENSIONS, SpanMap, SpanMapFeature, SpanMapFidelity,
    SpanMapKind, SpanMapSegment, TextRange, UnknownDiagnosticDirectivePolicy,
    UnknownSpanMapFidelity, UnknownSpanMapKind, is_supported_virtual_extension,
};
/// Snapshot/project/file diagnostics grouped by TypeScript category.
pub use diagnostics::{
    FileDiagnosticsResponse, ProjectDiagnosticsResponse, SnapshotDiagnosticsResponse,
};
/// Document identifiers and byte/UTF-16 positions used by many endpoints.
pub use document::{DocumentIdentifier, DocumentPosition};
/// Binary payload wrappers and print options.
pub use encoded::{EncodedPayload, PrintNodeOptions};
/// Opaque handles returned by Corsa.
pub use handles::{
    NodeHandle, ProjectHandle, SignatureHandle, SnapshotHandle, SymbolHandle, TypeHandle,
};
/// Fine-grained profiling hooks for request encode/transport/decode phases.
pub use profiling::{ApiProfileEvent, ApiProfilePhase, ApiProfiler, SharedProfiler};
/// Session wrapper that keeps a snapshot and default project alive.
pub use project_session::ProjectSession;
/// Common response payloads returned by the API.
pub use responses::{
    ConfigResponse, IndexInfo, InitializeResponse, JsDocTagInfo, ProjectResponse,
    SignatureResponse, SourceFileMetadata, SymbolResponse, TypePredicateResponse, TypeResponse,
    WellKnownSymbolsResponse,
};
/// Stable `corsa-bind`-owned semantic fact queries.
pub use semantics::{
    NameMeaning, SEMANTIC_QUERY_VERSION, SemanticQuery, SignatureKind, SymbolRef, TypeRef,
};
/// Auto-releasing snapshot wrapper.
pub use snapshot::ManagedSnapshot;
/// Source-file fields decoded out of the binary `getSourceFile` payload.
pub use source_file::{
    ContentMapping, EncodedSourceFile, MAX_SOURCE_FILE_PROTOCOL_VERSION,
    MIN_SOURCE_FILE_PROTOCOL_VERSION,
};
/// Higher-level checker probe models built from repeated project-session queries.
pub use type_probe::{TypeProbe, TypeProbeOptions};
