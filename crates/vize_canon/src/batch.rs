//! Batch TypeScript type checking for Vue SFC.
//!
//! This module provides batch type checking via `corsa-bind`.
//! It transforms Vue SFC files into pure TypeScript, materializes a virtual
//! project in a project-keyed namespace under `node_modules/.vize/canon/`, and
//! requests diagnostics from
//! Corsa's LSP instead of parsing CLI text output.

mod declaration_path;
mod error;
mod executor;
mod import_rewriter;
pub(crate) mod import_rewriter_alias;
#[cfg(test)]
mod import_rewriter_authored_vue_ts_tests;
#[cfg(test)]
mod import_rewriter_dts_tests;
#[cfg(test)]
mod import_rewriter_tests;
#[cfg(test)]
mod import_rewriter_type_tests;
#[cfg(test)]
mod import_rewriter_virtual_tests;
mod materialize_fs;
mod materialize_lock;
mod runtime_deps;
#[cfg(test)]
pub(crate) use runtime_deps::{VUE_RUNTIME_DOM_STUB_TYPES, write_vue_facade};
mod source_map;
mod source_policy;
mod type_checker;
pub(crate) mod virtual_project;
mod virtual_specifier_message;
mod virtual_ts;

pub use error::{CorsaError, CorsaNotFoundError, CorsaResult, PackageManager};
pub use executor::CorsaExecutor;
pub use import_rewriter::{ImportRewriter, ImportSourceMap, OffsetAdjustment, RewriteResult};
pub use source_map::{CompositeSourceMap, SfcBlockRange, SfcSourceMap};
pub use type_checker::{
    BatchTypeChecker, BatchTypeCheckerOptions, DeclarationEmitOptions, DeclarationEmitResult,
    DeclarationOutput, IncrementalCheckMetrics, TypeCheckResult, TypeChecker,
};
pub use virtual_project::{
    BatchTopologyMetrics, CONTENT_MAPPER_GENERATED_DIAGNOSTIC_CODE,
    CONTENT_MAPPER_SFC_PARSE_ERROR_CODE, CONTENT_MAPPER_VIRTUAL_EXTENSION, ContentMapperDiagnostic,
    ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives, ContentMapperSemanticLink,
    ContentMapperSpan, ContentMapperTransform, ContentMapperTransformOptions,
    ContentMapperUnusedExpectDiagnostic, OriginalPosition, PACKAGE_REACHABILITY_BUDGET_REVISION,
    PackageRouteReachability, ReachabilityOutcome, ReachabilityWork, TsconfigOwnershipCache,
    TsconfigOwnershipOptions, TsconfigSourceKind, VirtualFile, VirtualProject,
    VueDocumentVirtualTs, VueDocumentVirtualTsOptions, external_mirror_original_path,
    generate_vue_content_mapper_transform, generate_vue_content_mapper_transform_with_options,
    generate_vue_document_virtual_ts, generate_vue_document_virtual_ts_with_options,
    is_vue_runtime_support_specifier, project_virtual_lock_paths, project_virtual_root,
    scan_package_route_reachability, snapshot_tsconfig_compiler_options,
};
pub use virtual_specifier_message::{AUTHORED_VUE_TS_SENTINEL, restore_virtual_vue_specifiers};
pub use virtual_ts::VirtualTsGenerator;

pub(crate) use virtual_specifier_message::AUTHORED_VUE_TS_ALIAS_SENTINEL;

pub use crate::sfc_diagnostics::{SfcBlockType, sfc_block_fallback_offset};
use vize_carton::String;

/// Diagnostic reported by Corsa.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Original file path.
    pub file: std::path::PathBuf,
    /// Line number (0-based).
    pub line: u32,
    /// Column number (0-based).
    pub column: u32,
    /// Error message.
    pub message: String,
    /// TypeScript error code.
    pub code: Option<u32>,
    /// Severity (1=Error, 2=Warning, 3=Info, 4=Hint).
    pub severity: u8,
    /// SFC block type if applicable.
    pub block_type: Option<SfcBlockType>,
}
