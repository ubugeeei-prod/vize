//! Virtual project management for Corsa-backed type checking.
//!
//! This module materializes a mirrored TypeScript project in
//! `node_modules/.vize/canon/` so Corsa can type-check Vue SFCs together with
//! regular TypeScript sources, ambient declarations, and emitted `.d.ts` files.
//!
//! The implementation is split across submodules:
//!
//! - [`project`] — construction, configuration, and file registration.
//! - [`mapping`] — lookup and bidirectional position mapping.
//! - [`materialize`] — writing the virtual tree to disk.
//! - [`tsconfig_gen`] — generating the virtual `tsconfig.json`.
//! - [`build`] / [`passthrough`] — building self-contained registered files.
//! - [`vue_codegen`] / [`diagnostics`] — `.vue` virtual TS and SFC diagnostics.
//! - [`tsconfig_paths`] — tsconfig path resolution and JSONC parsing.

use std::path::PathBuf;

use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::{FxHashMap, String as CompactString};

use super::import_rewriter::ImportRewriter;
use super::source_map::CompositeSourceMap;
use super::{Diagnostic, SfcBlockType};
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions};

mod build;
pub use build::{VueDocumentVirtualTs, generate_vue_document_virtual_ts};
mod diagnostics;
mod jsx_codegen;
mod mapping;
mod materialize;
mod passthrough;
mod project;
mod tsconfig_gen;
mod tsconfig_paths;
mod vue_codegen;

#[cfg(test)]
mod tests;

pub(super) const AUTO_IMPORT_STUBS_FILE: &str = "__vize_auto_imports.d.ts";
pub(super) const VUE_MODULE_STUBS_FILE: &str = "__vize_vue_modules.d.ts";
/// Shared ambient helpers materialized once per program; generated `.vue.ts`
/// modules are emitted with their preamble hoisted into this file.
pub(super) const SHARED_HELPERS_FILE: &str = crate::virtual_ts::SHARED_PREAMBLE_FILE_NAME;

/// A virtual file in the project.
#[derive(Debug)]
pub struct VirtualFile {
    /// Generated or rewritten source code used by Corsa.
    pub content: CompactString,
    /// Source map for position mapping.
    pub source_map: CompositeSourceMap,
    /// Original file path.
    pub original_path: PathBuf,
    /// Materialized file path inside the virtual project.
    pub virtual_path: PathBuf,
}

/// Original position after mapping.
#[derive(Debug, Clone)]
pub struct OriginalPosition {
    /// Original file path.
    pub path: PathBuf,
    /// Line number (0-based).
    pub line: u32,
    /// Column number (0-based).
    pub column: u32,
    /// SFC block type if applicable.
    pub block_type: Option<SfcBlockType>,
}

/// Virtual project for Corsa-backed type checking.
pub struct VirtualProject {
    /// Project root directory.
    project_root: PathBuf,

    /// Virtual project root (`node_modules/.vize/canon`).
    virtual_root: PathBuf,

    /// Explicit tsconfig path, if one was provided by the caller.
    tsconfig_path: Option<PathBuf>,

    /// Whether the effective tsconfig asks TypeScript to report unused symbols.
    preserve_unused_diagnostics: bool,

    /// Global virtual TS options applied to every Vue file.
    virtual_ts_options: VirtualTsOptions,

    /// Internal check generation settings applied to every Vue file.
    virtual_ts_check_options: VirtualTsCheckOptions,

    /// Enable Vue 2.7 / Nuxt 2 Options API compatibility for virtual files.
    options_api: bool,
    legacy_vue2: bool,

    /// Opt-in type-checking of `.jsx`/`.tsx` Vue components (#1497). Default-off:
    /// when disabled, `.jsx`/`.tsx` are passed to TypeScript verbatim (React
    /// passthrough); when enabled, they are routed through the Vize JSX
    /// virtual-TS path.
    jsx_typecheck: bool,

    /// Configured Vue dialect from `vue.version` (default [`VueVersion::V3`]).
    ///
    /// Threaded in for future dialect-aware instance typing; plumbing only
    /// today, so it does not affect generated virtual TS yet.
    dialect: vize_carton::config::VueVersion,

    /// Template syntax compatibility used when parsing SFC templates.
    template_syntax: TemplateSyntaxMode,

    /// Virtual files keyed by materialized path.
    virtual_files: FxHashMap<PathBuf, VirtualFile>,

    /// Non-TS module files that must exist in the virtual mirror for TypeScript
    /// module resolution, keyed by materialized path.
    passthrough_files: FxHashMap<PathBuf, PathBuf>,

    /// Secondary index: original source path -> materialized (virtual) path.
    /// Keeps `find_by_original` / `map_to_virtual` O(1) instead of scanning
    /// every virtual file on each LSP position-mapping request.
    original_index: FxHashMap<PathBuf, PathBuf>,

    /// Original source text as registered, keyed by materialized (virtual)
    /// path. Retained here (rather than on the public `VirtualFile`) so
    /// position mapping can convert offsets to line/column without re-reading
    /// the file from disk on every request, and without changing the public
    /// `VirtualFile` API. This is also the exact text the source map's
    /// original offsets refer to (e.g. an editor's unsaved buffer), so it is
    /// more correct than re-reading live disk state.
    original_contents: FxHashMap<PathBuf, CompactString>,

    /// Parser diagnostics collected before Corsa runs.
    diagnostics: Vec<Diagnostic>,

    /// Import rewriter for `.vue` specifiers inside TypeScript sources.
    rewriter: ImportRewriter,
}
