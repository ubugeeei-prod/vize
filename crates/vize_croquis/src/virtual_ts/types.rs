//! Types for virtual TypeScript generation.

use vize_carton::{
    source_range::{SourceMap, SourceRange},
    CompactString,
};

use crate::import_resolver::ResolvedModule;
use vize_carton::String;

/// Output of virtual TypeScript generation.
#[derive(Debug, Clone)]
pub struct VirtualTsOutput {
    /// Generated TypeScript code
    pub content: String,
    /// Source map for position mapping
    pub source_map: SourceMap,
    /// Resolved external imports
    pub resolved_imports: Vec<ResolvedImport>,
    /// Diagnostics/warnings during generation
    pub diagnostics: Vec<GenerationDiagnostic>,
}

impl Default for VirtualTsOutput {
    fn default() -> Self {
        Self {
            content: String::default(),
            source_map: SourceMap::new(),
            resolved_imports: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

/// A resolved external import.
#[derive(Debug, Clone)]
pub struct ResolvedImport {
    /// Original import specifier
    pub specifier: CompactString,
    /// Resolved module
    pub module: ResolvedModule,
    /// Imported names
    pub names: Vec<CompactString>,
}

/// A diagnostic message from generation.
#[derive(Debug, Clone)]
pub struct GenerationDiagnostic {
    /// Message text
    pub message: CompactString,
    /// Source range (if applicable)
    pub range: Option<SourceRange>,
    /// Severity level
    pub severity: DiagnosticSeverity,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Error - generation failed
    Error,
    /// Warning - generation succeeded with issues
    Warning,
    /// Info - informational message
    Info,
}

/// Configuration for virtual TypeScript generation.
#[derive(Debug, Clone, Default)]
pub struct VirtualTsConfig {
    /// Generic type parameter from `<script setup generic="T">`
    /// (Can be overridden, but prefer extracting from ScopeChain)
    pub generic: Option<CompactString>,
    /// Whether this is async setup
    pub is_async: bool,
    /// Script block offset in the original SFC
    pub script_offset: u32,
    /// Template block offset in the original SFC
    pub template_offset: u32,
}
