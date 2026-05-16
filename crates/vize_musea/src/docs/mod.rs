//! Documentation generation for Art files.
//!
//! This module generates Markdown documentation from Art descriptors,
//! including component catalogs, variant listings, and searchable indexes.
//!
//! # Example
//!
//! ```rust
//! use vize_carton::Bump;
//! use vize_musea::{parse_art, ArtParseOptions};
//! use vize_musea::docs::{generate_component_doc, DocOptions};
//!
//! let allocator = Bump::new();
//! let source = r#"
//! <art title="Button" description="A button component" category="atoms">
//!   <variant name="Primary" default>
//!     <Button>Click me</Button>
//!   </variant>
//! </art>
//! "#;
//!
//! let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
//! let output = generate_component_doc(&art, &DocOptions::default());
//! println!("{}", output.markdown);
//! ```

mod catalog;
mod markdown;

pub use catalog::{generate_catalog, generate_category_index, generate_tags_index, CatalogEntry};
pub use markdown::{generate_component_doc, generate_variant_doc};

use serde::{Deserialize, Serialize};
use vize_carton::String;

/// Options for documentation generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocOptions {
    /// Include source code snippets in documentation.
    #[serde(default)]
    pub include_source: bool,

    /// Include variant templates in documentation.
    #[serde(default = "default_true")]
    pub include_templates: bool,

    /// Include metadata (tags, category, status) in documentation.
    #[serde(default = "default_true")]
    pub include_metadata: bool,

    /// Include table of contents for components with many variants.
    #[serde(default = "default_true")]
    pub include_toc: bool,

    /// Minimum number of variants to show table of contents.
    #[serde(default = "default_toc_threshold")]
    pub toc_threshold: usize,

    /// Base path for component links.
    #[serde(default)]
    pub base_path: String,

    /// Custom title for the documentation.
    #[serde(default)]
    pub title: Option<String>,

    /// Include timestamp in generated documentation.
    #[serde(default)]
    pub include_timestamp: bool,
}

fn default_true() -> bool {
    true
}

fn default_toc_threshold() -> usize {
    5
}

/// Output of documentation generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocOutput {
    /// Generated Markdown content.
    pub markdown: String,

    /// Suggested filename for the documentation.
    pub filename: String,

    /// Title extracted from the art file.
    pub title: String,

    /// Category if present.
    pub category: Option<String>,

    /// Number of variants documented.
    pub variant_count: usize,
}

/// Output of catalog generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogOutput {
    /// Generated Markdown content.
    pub markdown: String,

    /// Suggested filename.
    pub filename: String,

    /// Number of components in catalog.
    pub component_count: usize,

    /// Categories found.
    pub categories: Vec<String>,

    /// All tags found.
    pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{generate_catalog, generate_component_doc, CatalogEntry, DocOptions};
    use crate::{parse_art, ArtParseOptions};
    use vize_carton::Bump;

    #[test]
    fn test_generate_component_doc() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button" description="A versatile button" category="atoms" tags="ui,input">
  <variant name="Primary" default>
    <Button variant="primary">Click</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Click</Button>
  </variant>
</art>
"#;

        let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
        let output = generate_component_doc(&art, &DocOptions::default());

        insta::assert_debug_snapshot!(output);
    }

    #[test]
    fn test_generate_catalog() {
        let allocator = Bump::new();
        let sources = [
            r#"<art title="Button" category="atoms"><variant name="Default"><div></div></variant></art>"#,
            r#"<art title="Card" category="molecules"><variant name="Default"><div></div></variant></art>"#,
        ];

        let entries: Vec<_> = sources
            .iter()
            .map(|s| {
                let art = parse_art(&allocator, s, ArtParseOptions::default()).unwrap();
                CatalogEntry::from_descriptor(&art, "")
            })
            .collect();

        let output = generate_catalog(&entries, &DocOptions::default());

        insta::assert_debug_snapshot!(output);
    }
}
