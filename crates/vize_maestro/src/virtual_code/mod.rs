//! Virtual Code Layer for embedded language support.
//!
//! This module implements the Virtual Code architecture inspired by Volar,
//! which transforms Vue SFC files into virtual documents for each embedded language.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]
//!
//! ## Architecture
//!
//! ```text
//! .vue SFC File
//!     │
//!     ▼
//! ┌─────────────────────────────────────┐
//! │ VirtualCodeGenerator                │
//! │ (SFC → Virtual Documents)           │
//! └─────────────────────────────────────┘
//!     │
//!     ├─► Template Virtual (.vue.__template.ts)
//!     │   - Extracts template expressions
//!     │   - Generates TypeScript for type checking
//!     │
//!     ├─► Script Virtual (.vue.__script.ts)
//!     │   - Preserves script content
//!     │   - Exports bindings for template
//!     │
//!     └─► Style Virtual (.vue.__style_N.css)
//!         - Preserves style content
//!         - One per <style> block
//! ```

mod generator;
mod script_code;
mod source_map;
mod style_code;
mod template_code;

pub use generator::{
    find_art_block_at_offset, find_block_at_offset, ArtCursorPosition, ArtVariantInfo,
    BatchVirtualCodeGenerator, BlockType, VirtualCodeGenerator,
};
pub use script_code::{extract_simple_bindings, ScriptCodeGenerator};
pub use source_map::{MappingData, MappingFeatures, SourceMap, SourceMapping};
pub use style_code::{StyleCodeGenerator, StyleMetadata};
pub use template_code::{
    extract_expressions, ExpressionKind, TemplateCodeGenerator, TemplateExpression,
};

/// Virtual language types supported by the LSP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualLanguage {
    /// Template block - generates TypeScript for type checking
    Template,
    /// Script block - TypeScript/JavaScript
    Script,
    /// Script setup block - treated separately for binding analysis
    ScriptSetup,
    /// Style block - CSS/SCSS/Less
    Style,
}

impl VirtualLanguage {
    /// Get the file extension for this virtual language.
    pub fn extension(&self) -> &'static str {
        match self {
            VirtualLanguage::Template => "ts",
            VirtualLanguage::Script => "ts",
            VirtualLanguage::ScriptSetup => "ts",
            VirtualLanguage::Style => "css",
        }
    }

    /// Get the language ID for LSP.
    pub fn language_id(&self) -> &'static str {
        match self {
            VirtualLanguage::Template => "typescript",
            VirtualLanguage::Script => "typescript",
            VirtualLanguage::ScriptSetup => "typescript",
            VirtualLanguage::Style => "css",
        }
    }
}

/// A virtual document generated from an SFC block.
#[derive(Debug, Clone)]
pub struct VirtualDocument {
    /// Virtual document URI (e.g., "file.vue.__template.ts")
    pub uri: String,
    /// Generated content
    pub content: String,
    /// Virtual language type
    pub language: VirtualLanguage,
    /// Source mappings for position translation
    pub source_map: SourceMap,
}

impl VirtualDocument {
    /// Create a new virtual document.
    pub fn new(uri: String, content: String, language: VirtualLanguage) -> Self {
        Self {
            uri,
            content,
            language,
            source_map: SourceMap::new(),
        }
    }

    /// Create with source mappings.
    pub fn with_mappings(
        uri: String,
        content: String,
        language: VirtualLanguage,
        mappings: Vec<SourceMapping>,
    ) -> Self {
        Self {
            uri,
            content,
            language,
            source_map: SourceMap::from_mappings(mappings),
        }
    }
}

/// Collection of virtual documents for an SFC.
#[derive(Debug, Default)]
pub struct VirtualDocuments {
    /// Template virtual document
    pub template: Option<VirtualDocument>,
    /// Script virtual document
    pub script: Option<VirtualDocument>,
    /// Script setup virtual document
    pub script_setup: Option<VirtualDocument>,
    /// Style virtual documents (one per <style> block)
    pub styles: Vec<VirtualDocument>,
}

impl VirtualDocuments {
    /// Create empty virtual documents.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all virtual documents as a vector.
    pub fn all(&self) -> Vec<&VirtualDocument> {
        let mut docs = Vec::new();
        if let Some(ref t) = self.template {
            docs.push(t);
        }
        if let Some(ref s) = self.script {
            docs.push(s);
        }
        if let Some(ref ss) = self.script_setup {
            docs.push(ss);
        }
        for style in &self.styles {
            docs.push(style);
        }
        docs
    }

    /// Find the virtual document containing the given source offset.
    pub fn find_by_source_offset(&self, offset: u32) -> Option<(&VirtualDocument, u32)> {
        // Check each virtual document's source map
        for doc in self.all() {
            if let Some(gen_offset) = doc.source_map.to_generated(offset) {
                return Some((doc, gen_offset));
            }
        }
        None
    }
}

/// Source range in a document.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceRange {
    /// Start byte offset
    pub start: u32,
    /// End byte offset
    pub end: u32,
}

impl SourceRange {
    /// Create a new source range.
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Check if this range contains the given offset.
    pub fn contains(&self, offset: u32) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Get the length of this range.
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Check if this range is empty.
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

impl From<vize_relief::SourceLocation> for SourceRange {
    fn from(loc: vize_relief::SourceLocation) -> Self {
        Self {
            start: loc.start.offset,
            end: loc.end.offset,
        }
    }
}

impl From<&vize_relief::SourceLocation> for SourceRange {
    fn from(loc: &vize_relief::SourceLocation) -> Self {
        Self {
            start: loc.start.offset,
            end: loc.end.offset,
        }
    }
}
