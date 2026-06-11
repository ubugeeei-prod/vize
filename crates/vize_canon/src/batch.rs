//! Batch TypeScript type checking for Vue SFC.
//!
//! This module provides batch type checking via `corsa-bind`.
//! It transforms Vue SFC files into pure TypeScript, materializes a virtual
//! project in `node_modules/.vize/canon/`, and requests diagnostics from
//! Corsa's LSP instead of parsing CLI text output.

mod error;
mod executor;
mod import_rewriter;
mod materialize_fs;
mod runtime_deps;
mod source_map;
mod type_checker;
mod virtual_project;
mod virtual_ts;

pub use error::{CorsaError, CorsaNotFoundError, CorsaResult, PackageManager};
pub use executor::CorsaExecutor;
pub use import_rewriter::{ImportRewriter, ImportSourceMap, OffsetAdjustment, RewriteResult};
pub use source_map::{CompositeSourceMap, SfcBlockRange, SfcSourceMap};
pub use type_checker::{
    BatchTypeChecker, BatchTypeCheckerOptions, DeclarationEmitOptions, DeclarationEmitResult,
    DeclarationOutput, TypeCheckResult, TypeChecker,
};
pub use virtual_project::{
    OriginalPosition, VirtualFile, VirtualProject, VueDocumentVirtualTs,
    generate_vue_document_virtual_ts,
};
pub use virtual_ts::VirtualTsGenerator;

use vize_carton::String;

/// SFC block type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfcBlockType {
    Template,
    Script,
    ScriptSetup,
    Style,
}

impl SfcBlockType {
    /// The SFC block name as it appears in a `.vue` file / `@vue/compiler-sfc`
    /// descriptor (`scriptSetup`, `script`, `template`, `style`).
    pub fn block_name(self) -> &'static str {
        match self {
            SfcBlockType::Template => "template",
            SfcBlockType::Script => "script",
            SfcBlockType::ScriptSetup => "scriptSetup",
            SfcBlockType::Style => "style",
        }
    }
}

/// Best-effort fallback byte offset for SFC diagnostics that ship without a
/// `loc`. Returns the start of the most relevant block (`<script setup>`, then
/// `<script>`, then `<template>`) so the diagnostic lands somewhere clickable
/// instead of at file offset 0.
///
/// Shared by the canon batch pipeline, the `corsa_server` transport, and the
/// maestro diagnostic collectors so the fallback location is computed in one
/// place (#1389). Returns `None` when the descriptor has none of those blocks;
/// callers fall back to `(0, _)`.
pub fn sfc_block_fallback_offset(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
) -> Option<(usize, SfcBlockType)> {
    if let Some(setup) = descriptor.script_setup.as_ref() {
        return Some((setup.loc.start, SfcBlockType::ScriptSetup));
    }
    if let Some(script) = descriptor.script.as_ref() {
        return Some((script.loc.start, SfcBlockType::Script));
    }
    if let Some(template) = descriptor.template.as_ref() {
        return Some((template.loc.start, SfcBlockType::Template));
    }
    None
}

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
