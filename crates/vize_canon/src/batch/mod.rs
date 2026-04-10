//! Batch TypeScript type checking for Vue SFC.
//!
//! This module provides batch type checking via `corsa-bind`.
//! It transforms Vue SFC files into pure TypeScript, materializes a virtual
//! project in `node_modules/.vize/canon/`, and requests diagnostics from
//! Corsa's LSP instead of parsing CLI text output.

mod error;
mod executor;
mod import_rewriter;
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
pub use virtual_project::{OriginalPosition, VirtualFile, VirtualProject};
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
