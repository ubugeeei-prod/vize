//! Virtual TypeScript generator implementation.
//!
//! This module is split into:
//! - `script`: Script setup generation, imports, compiler macros
//! - `template`: Template AST traversal and expression emission

mod script;
mod template;

use std::path::Path;

use vize_carton::{
    source_range::{MappingData, SourceMap, SourceMapping, SourceRange},
    CompactString,
};
use vize_relief::ast::{
    DirectiveNode, ElementNode, ExpressionNode, ForNode, IfNode, InterpolationNode, PropNode,
    RootNode, TemplateChildNode,
};
use vize_relief::BindingType;

use crate::analysis::BindingMetadata;
use crate::import_resolver::ImportResolver;
use crate::macros::MacroTracker;
use crate::scope::{ScopeChain, ScopeData, ScopeKind};
use crate::script_parser::ScriptParseResult;
use crate::types::TypeResolver;

use super::types::{GenerationDiagnostic, ResolvedImport, VirtualTsConfig, VirtualTsOutput};
use vize_carton::String;

/// Virtual TypeScript generator.
///
/// Generates TypeScript code from Vue SFC components for type checking.
/// Supports:
/// - Script setup with defineProps/defineEmits
/// - Generic type parameters (`<script setup generic="T">`)
/// - Template expressions with proper typing
/// - External type imports resolution
pub struct VirtualTsGenerator {
    /// Type resolver for inline types
    type_resolver: TypeResolver,
    /// Import resolver for external types
    import_resolver: Option<ImportResolver>,
    /// Generated output buffer
    output: String,
    /// Source mappings
    mappings: Vec<SourceMapping>,
    /// Current output offset
    gen_offset: u32,
    /// Expression counter for unique names
    expr_counter: u32,
    /// Block offset in original SFC
    block_offset: u32,
    /// Resolved imports
    resolved_imports: Vec<ResolvedImport>,
    /// Generation diagnostics
    diagnostics: Vec<GenerationDiagnostic>,
    /// Current indentation level
    indent_level: usize,
}

impl VirtualTsGenerator {
    /// Create a new generator.
    pub fn new() -> Self {
        Self {
            type_resolver: TypeResolver::new(),
            import_resolver: None,
            output: String::with_capacity(4096),
            mappings: Vec::with_capacity(64),
            gen_offset: 0,
            expr_counter: 0,
            block_offset: 0,
            resolved_imports: Vec::new(),
            diagnostics: Vec::new(),
            indent_level: 0,
        }
    }

    /// Create with an import resolver.
    pub fn with_import_resolver(mut self, resolver: ImportResolver) -> Self {
        self.import_resolver = Some(resolver);
        self
    }

    /// Set the type resolver.
    pub fn with_type_resolver(mut self, resolver: TypeResolver) -> Self {
        self.type_resolver = resolver;
        self
    }

    /// Reset state for a new generation.
    fn reset(&mut self) {
        self.output.clear();
        self.mappings.clear();
        self.gen_offset = 0;
        self.expr_counter = 0;
        self.block_offset = 0;
        self.resolved_imports.clear();
        self.diagnostics.clear();
        self.indent_level = 0;
    }

    /// Create the output from current state.
    fn create_output(&mut self) -> VirtualTsOutput {
        let mut source_map = SourceMap::from_mappings(std::mem::take(&mut self.mappings));
        source_map.set_block_offset(self.block_offset);

        VirtualTsOutput {
            content: std::mem::take(&mut self.output),
            source_map,
            resolved_imports: std::mem::take(&mut self.resolved_imports),
            diagnostics: std::mem::take(&mut self.diagnostics),
        }
    }

    /// Write a string to output.
    fn write(&mut self, s: &str) {
        self.output.push_str(s);
        self.gen_offset += s.len() as u32;
    }

    /// Write a line to output (no indentation).
    fn write_line(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
        self.gen_offset += s.len() as u32 + 1;
    }

    /// Write a line with proper indentation.
    fn emit_line(&mut self, s: &str) {
        let indent = "  ".repeat(self.indent_level);
        self.output.push_str(&indent);
        self.output.push_str(s);
        self.output.push('\n');
        self.gen_offset += indent.len() as u32 + s.len() as u32 + 1;
    }

    /// Write a generated line with proper indentation and no intermediate string.
    fn emit_generated_line(&mut self, f: impl FnOnce(&mut String)) {
        let start = self.output.len();
        for _ in 0..self.indent_level {
            self.output.push_str("  ");
        }
        f(&mut self.output);
        self.output.push('\n');
        self.gen_offset += (self.output.len() - start) as u32;
    }
}

impl Default for VirtualTsGenerator {
    fn default() -> Self {
        Self::new()
    }
}
