//! # vize_canon
//!
//! Canon - The standard of correctness for Vize.
//! TypeScript type checker for Vue.js Single File Components.
//!
//! ## Name Origin
//!
//! **Canon** (/ˈkænən/) in art refers to a set of ideal proportions or standards
//! that define perfection. Just as classical sculptors followed canons to
//! achieve harmonious proportions, `vize_canon` enforces type correctness
//! as the standard for Vue SFC code.
//!
//! ## Architecture
//!
//! ### Batch Type Checking (via corsa-bind)
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                         vize Process                            │
//! │                                                                 │
//! │  Project                    Virtual Project                     │
//! │  ┌──────────────┐          ┌──────────────────────────┐        │
//! │  │ src/         │          │ .vize/canon/projects/key │        │
//! │  │ ├─ App.vue   │ ──────▶  │ ├─ src/                  │        │
//! │  │ ├─ utils.ts  │          │ │  ├─ App.vue.ts         │        │
//! │  │ └─ ...       │          │ │  ├─ utils.ts           │        │
//! │  └──────────────┘          │ │  └─ ...                │        │
//! │                            │ └─ tsconfig.json         │        │
//! │                            └──────────────────────────┘        │
//! │                                       │                         │
//! │                                       ▼                         │
//! │                            ┌──────────────────────────┐        │
//! │                            │  Corsa ProjectSession    │        │
//! │                            │  sync msgpack stdio      │        │
//! │                            └──────────┬───────────────┘        │
//! │                                       │                         │
//! │                                       ▼                         │
//! │                            ┌──────────────────────────┐        │
//! │                            │ Diagnostics + SourceMap  │        │
//! │                            │ → Map to original SFC    │        │
//! │                            └──────────────────────────┘        │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

extern crate vize_s0 as vize_carton;

mod checker;
mod context;
#[cfg(feature = "native")]
pub mod corsa_session_cache;
mod diagnostic;
#[cfg(feature = "native")]
mod file_uri;
pub mod intelligence;
mod options_api_setup_spread;
pub mod package_route;
mod script_parse;
mod sfc_diagnostics;
pub mod sfc_typecheck;
pub mod source_map;
mod types;
pub mod virtual_ts;

/// Legacy Vue (v0.10 / v0.11 / v1 / v2) type-checking surface. Gated behind the `legacy`
/// feature and dropped from the default Vue 3 build; opt-in only.
#[cfg(feature = "legacy")]
pub mod legacy;

// Batch type checking module (requires native feature)
#[cfg(feature = "native")]
pub mod batch;
#[cfg(feature = "native")]
pub mod corsa_bridge;

#[cfg(feature = "native")]
pub mod lsp_client;

#[cfg(feature = "native")]
pub use lsp_client as corsa_client;

#[cfg(all(feature = "native", unix))]
pub mod corsa_server;

#[cfg(feature = "native")]
pub mod typecheck_service;

#[cfg(all(test, feature = "native"))]
mod template_ref_registry_scope;

#[cfg(all(test, feature = "native"))]
mod tests;

#[cfg(all(test, feature = "native"))]
mod type_only_import_anchors;

pub use checker::TypeChecker;
pub use context::{Binding, BindingKind, Import, Prop, TypeContext};
pub use diagnostic::{TypeDiagnostic, TypeErrorCode, TypeSeverity};
pub use sfc_diagnostics::{SfcBlockType, sfc_block_fallback_offset};

// Re-export Locale for i18n support
pub use intelligence::{
    Completion, CompletionKind as IntelCompletionKind, CursorContext, Diagnostic,
    DiagnosticSeverity, HoverInfo, Location, TypeIntelligence,
};
pub use package_route::{
    PackageResolutionContext, PackageResolutionMode, PackageRoute, PackageRouteBinding,
    PackageRouteLookup, PackageRouteMetrics, PackageRouteResolver, PackageRouteSource,
    PackageSourceOptions,
};
pub use sfc_typecheck::{
    SfcRelatedLocation, SfcTypeCheckOptions, SfcTypeCheckResult, SfcTypeDiagnostic,
    SfcTypeSeverity, type_check_sfc, type_check_sfc_with_legacy_vue2,
    type_check_sfc_with_options_api,
};
pub use source_map::{
    Mapping, MappingFlags, MappingKind, Position, SourceMap, Span, offset_to_position,
    position_to_offset,
};
pub use types::{CompletionItem, CompletionKind, TypeInfo, TypeKind};
pub use vize_s0::i18n::Locale;

#[cfg(feature = "native")]
pub use corsa_bridge::{
    CorsaBridge, CorsaBridgeConfig, CorsaBridgeError, CorsaMaterializedMappingKind,
    CorsaMaterializedSource, CorsaScriptVirtualDocumentRequest, CorsaVueVirtualDependency,
    CorsaVueVirtualDocument, CorsaVueVirtualDocumentOptions, LspCompletionItem, LspCompletionList,
    LspCompletionResponse, LspDefinitionResponse, LspDiagnostic, LspDocumentation, LspHover,
    LspHoverContents, LspLocation, LspLocationLink, LspMarkedString, LspMarkupContent,
    LspParameterInformation, LspParameterLabel, LspPosition, LspRange, LspSignatureHelp,
    LspSignatureInformation, VIRTUAL_URI_SCHEME,
};

// Re-export batch type checker
#[cfg(feature = "native")]
pub use batch::{
    BatchTopologyMetrics, BatchTypeChecker, BatchTypeCheckerOptions,
    CONTENT_MAPPER_GENERATED_DIAGNOSTIC_CODE, CONTENT_MAPPER_SFC_PARSE_ERROR_CODE,
    CONTENT_MAPPER_VIRTUAL_EXTENSION, ContentMapperDiagnostic, ContentMapperDiagnosticDirective,
    ContentMapperDiagnosticDirectives, ContentMapperSemanticLink, ContentMapperSpan,
    ContentMapperTransform, ContentMapperTransformOptions, ContentMapperUnusedExpectDiagnostic,
    CorsaError, CorsaExecutor, CorsaNotFoundError, DeclarationEmitOptions, DeclarationEmitResult,
    DeclarationOutput, Diagnostic as BatchDiagnostic, ImportRewriter, ImportSourceMap,
    IncrementalCheckMetrics, PackageManager, TypeCheckResult as BatchTypeCheckResult,
    TypeChecker as BatchTypeCheckerTrait, VirtualFile, VirtualProject, VirtualTsGenerator,
    generate_vue_content_mapper_transform, generate_vue_content_mapper_transform_with_options,
    project_virtual_lock_paths, project_virtual_root, snapshot_tsconfig_compiler_options,
};

#[cfg(feature = "native")]
pub use typecheck_service::{
    SfcDiagnostic, SfcDiagnosticSeverity, SfcRelatedInfo,
    SfcTypeCheckResult as CorsaTypeCheckResult, TypeCheckService, TypeCheckServiceOptions,
};

#[cfg(all(feature = "native", unix))]
pub use corsa_server::{
    CheckParams, CheckResult as CorsaServerCheckResult, CorsaServer,
    Diagnostic as CorsaServerDiagnostic, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    ServerConfig,
};

/// Check result from the type checker.
#[derive(Debug, Clone, Default)]
pub struct CheckResult {
    /// Type diagnostics (errors and warnings).
    pub diagnostics: Vec<TypeDiagnostic>,
    /// Error count.
    pub error_count: usize,
    /// Warning count.
    pub warning_count: usize,
}

impl CheckResult {
    /// Create a new empty check result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if there are errors.
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Check if there are any diagnostics.
    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Add a diagnostic.
    pub fn add_diagnostic(&mut self, diagnostic: TypeDiagnostic) {
        match diagnostic.severity {
            TypeSeverity::Error => self.error_count += 1,
            TypeSeverity::Warning => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }
}
