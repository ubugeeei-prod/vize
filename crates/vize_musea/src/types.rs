//! Type definitions for vize_musea.
//!
//! This module contains the core data structures for representing
//! Art files (*.art.vue) and their components.
//!
//! All types are designed for zero-copy parsing with arena allocation.

use serde::{Deserialize, Serialize};
use vize_carton::{Bump, FxHashMap, String, ToCompactString, Vec as BumpVec};

/// Parsed Art file descriptor.
///
/// Uses arena allocation for all collections.
/// String data is borrowed directly from source.
#[derive(Debug)]
pub struct ArtDescriptor<'a> {
    /// Source filename
    pub filename: &'a str,

    /// Original source code (borrowed)
    pub source: &'a str,

    /// Art metadata from `<art>` block attributes
    pub metadata: ArtMetadata<'a>,

    /// Variant definitions from `<variant>` blocks (arena-allocated)
    pub variants: BumpVec<'a, ArtVariant<'a>>,

    /// Script setup block (if present)
    pub script_setup: Option<ArtScriptBlock<'a>>,

    /// Regular script block (if present)
    pub script: Option<ArtScriptBlock<'a>>,

    /// Style blocks (arena-allocated)
    pub styles: BumpVec<'a, ArtStyleBlock<'a>>,
}

/// Art metadata extracted from `<art>` block attributes.
#[derive(Debug)]
pub struct ArtMetadata<'a> {
    /// Display title (required) - borrowed from source
    pub title: &'a str,

    /// Description text - borrowed from source
    pub description: Option<&'a str>,

    /// Path to the target component - borrowed from source
    pub component: Option<&'a str>,

    /// Category for organization - borrowed from source
    pub category: Option<&'a str>,

    /// Tags for filtering/searching (arena-allocated)
    pub tags: BumpVec<'a, &'a str>,

    /// Status indicator
    pub status: ArtStatus,

    /// Display order (lower = first)
    pub order: Option<u32>,
}

/// Art status indicator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtStatus {
    /// Work in progress
    Draft,
    /// Ready for use
    #[default]
    Ready,
    /// No longer recommended
    Deprecated,
}

/// A single variant definition from `<variant>` block.
#[derive(Debug)]
pub struct ArtVariant<'a> {
    /// Variant name (required) - borrowed from source
    pub name: &'a str,

    /// Template content inside `<variant>` - borrowed from source
    pub template: &'a str,

    /// Whether this is the default variant
    pub is_default: bool,

    /// Props/args override for this variant
    pub args: FxHashMap<&'a str, serde_json::Value>,

    /// Viewport configuration for VRT
    pub viewport: Option<ViewportConfig>,

    /// Skip this variant in VRT
    pub skip_vrt: bool,

    /// Source location (byte offsets for fast access)
    pub loc: Option<SourceLocation>,
}

/// Script block in Art file.
#[derive(Debug)]
pub struct ArtScriptBlock<'a> {
    /// Script content - borrowed from source
    pub content: &'a str,

    /// Language (ts, js, tsx, jsx)
    pub lang: Option<&'a str>,

    /// Whether this is a setup script
    pub setup: bool,

    /// Source location
    pub loc: Option<SourceLocation>,
}

/// Style block in Art file.
#[derive(Debug)]
pub struct ArtStyleBlock<'a> {
    /// Style content - borrowed from source
    pub content: &'a str,

    /// Language (css, scss, less, etc.)
    pub lang: Option<&'a str>,

    /// Whether scoped
    pub scoped: bool,

    /// Source location
    pub loc: Option<SourceLocation>,
}

/// Viewport configuration for VRT.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewportConfig {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Device scale factor (default: 1.0)
    pub device_scale_factor: Option<f32>,
}

/// Source location information (byte offsets for fast access).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Start byte offset
    pub start: u32,
    /// End byte offset
    pub end: u32,
    /// Start line (1-indexed, computed lazily)
    pub start_line: u32,
    /// Start column (0-indexed)
    pub start_column: u32,
}

/// Parse options for Art files.
#[derive(Debug, Clone, Default)]
pub struct ArtParseOptions {
    /// Filename for error messages
    pub filename: String,
}

/// Parse result containing descriptor or errors.
pub type ArtParseResult<'a> = Result<ArtDescriptor<'a>, ArtParseError>;

/// Error type for Art parsing.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ArtParseError {
    #[error("Missing required 'title' attribute in <art> block")]
    MissingTitle,

    #[error("Missing required 'name' attribute in <variant> block at line {line}")]
    MissingVariantName { line: u32 },

    #[error("No <art> block found in file")]
    NoArtBlock,

    #[error("Invalid attribute value for '{attr}': {message}")]
    InvalidAttribute { attr: String, message: String },

    #[error("Parse error at line {line}: {message}")]
    ParseError { line: u32, message: String },
}

/// Output of Storybook CSF transformation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsfOutput {
    /// Generated CSF code
    pub code: String,
    /// Suggested filename (e.g., "Button.stories.ts")
    pub filename: String,
}

impl<'a> ArtDescriptor<'a> {
    /// Create a new descriptor with arena allocation.
    #[inline]
    pub fn new(allocator: &'a Bump, filename: &'a str, source: &'a str) -> Self {
        Self {
            filename,
            source,
            metadata: ArtMetadata::new(allocator),
            variants: BumpVec::new_in(allocator),
            script_setup: None,
            script: None,
            styles: BumpVec::new_in(allocator),
        }
    }

    /// Get the default variant, or the first one if none is marked default.
    #[inline]
    pub fn default_variant(&self) -> Option<&ArtVariant<'a>> {
        self.variants
            .iter()
            .find(|v| v.is_default)
            .or_else(|| self.variants.first())
    }
}

impl<'a> ArtMetadata<'a> {
    /// Create default metadata with arena allocation.
    #[inline]
    pub fn new(allocator: &'a Bump) -> Self {
        Self {
            title: "",
            description: None,
            component: None,
            category: None,
            tags: BumpVec::new_in(allocator),
            status: ArtStatus::default(),
            order: None,
        }
    }
}

impl<'a> ArtVariant<'a> {
    /// Create a new variant.
    #[inline]
    pub fn new(name: &'a str, template: &'a str) -> Self {
        Self {
            name,
            template,
            is_default: false,
            args: FxHashMap::default(),
            viewport: None,
            skip_vrt: false,
            loc: None,
        }
    }
}

impl Default for ViewportConfig {
    #[inline]
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            device_scale_factor: Some(1.0),
        }
    }
}

impl SourceLocation {
    /// Create a new source location.
    #[inline]
    pub const fn new(start: u32, end: u32, start_line: u32, start_column: u32) -> Self {
        Self {
            start,
            end,
            start_line,
            start_column,
        }
    }
}

// ============================================================================
// Serialization support for WASM/NAPI
// ============================================================================

/// Owned version of ArtDescriptor for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtDescriptorOwned {
    pub filename: String,
    pub source: String,
    pub metadata: ArtMetadataOwned,
    pub variants: Vec<ArtVariantOwned>,
    pub script_setup: Option<ArtScriptBlockOwned>,
    pub script: Option<ArtScriptBlockOwned>,
    pub styles: Vec<ArtStyleBlockOwned>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtMetadataOwned {
    pub title: String,
    pub description: Option<String>,
    pub component: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub status: ArtStatus,
    pub order: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtVariantOwned {
    pub name: String,
    pub template: String,
    pub is_default: bool,
    pub args: FxHashMap<String, serde_json::Value>,
    pub viewport: Option<ViewportConfig>,
    pub skip_vrt: bool,
    pub loc: Option<SourceLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtScriptBlockOwned {
    pub content: String,
    pub lang: Option<String>,
    pub setup: bool,
    pub loc: Option<SourceLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtStyleBlockOwned {
    pub content: String,
    pub lang: Option<String>,
    pub scoped: bool,
    pub loc: Option<SourceLocation>,
}

impl<'a> ArtDescriptor<'a> {
    /// Convert to owned version for serialization.
    pub fn into_owned(self) -> ArtDescriptorOwned {
        ArtDescriptorOwned {
            filename: self.filename.to_compact_string(),
            source: self.source.to_compact_string(),
            metadata: self.metadata.into_owned(),
            variants: self.variants.into_iter().map(|v| v.into_owned()).collect(),
            script_setup: self.script_setup.map(|s| s.into_owned()),
            script: self.script.map(|s| s.into_owned()),
            styles: self.styles.into_iter().map(|s| s.into_owned()).collect(),
        }
    }
}

impl<'a> ArtMetadata<'a> {
    /// Convert to owned version.
    pub fn into_owned(self) -> ArtMetadataOwned {
        ArtMetadataOwned {
            title: self.title.to_compact_string(),
            description: self.description.map(|s| s.to_compact_string()),
            component: self.component.map(|s| s.to_compact_string()),
            category: self.category.map(|s| s.to_compact_string()),
            tags: self
                .tags
                .into_iter()
                .map(|s| s.to_compact_string())
                .collect(),
            status: self.status,
            order: self.order,
        }
    }
}

impl<'a> ArtVariant<'a> {
    /// Convert to owned version.
    pub fn into_owned(self) -> ArtVariantOwned {
        ArtVariantOwned {
            name: self.name.to_compact_string(),
            template: self.template.to_compact_string(),
            is_default: self.is_default,
            args: self
                .args
                .into_iter()
                .map(|(k, v)| (k.to_compact_string(), v))
                .collect(),
            viewport: self.viewport,
            skip_vrt: self.skip_vrt,
            loc: self.loc,
        }
    }
}

impl<'a> ArtScriptBlock<'a> {
    /// Convert to owned version.
    pub fn into_owned(self) -> ArtScriptBlockOwned {
        ArtScriptBlockOwned {
            content: self.content.to_compact_string(),
            lang: self.lang.map(|s| s.to_compact_string()),
            setup: self.setup,
            loc: self.loc,
        }
    }
}

impl<'a> ArtStyleBlock<'a> {
    /// Convert to owned version.
    pub fn into_owned(self) -> ArtStyleBlockOwned {
        ArtStyleBlockOwned {
            content: self.content.to_compact_string(),
            lang: self.lang.map(|s| s.to_compact_string()),
            scoped: self.scoped,
            loc: self.loc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ArtDescriptor, ArtStatus, ViewportConfig};
    use vize_carton::Bump;

    #[test]
    fn test_art_descriptor_new() {
        let allocator = Bump::new();
        let desc = ArtDescriptor::new(&allocator, "test.art.vue", "<art></art>");
        assert_eq!(desc.filename, "test.art.vue");
        assert!(desc.variants.is_empty());
    }

    #[test]
    fn test_art_status_default() {
        assert_eq!(ArtStatus::default(), ArtStatus::Ready);
    }

    #[test]
    fn test_viewport_default() {
        let vp = ViewportConfig::default();
        assert_eq!(vp.width, 1280);
        assert_eq!(vp.height, 720);
    }
}
