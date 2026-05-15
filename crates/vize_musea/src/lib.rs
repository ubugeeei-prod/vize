//! # vize_musea
//!
//! Musea - Component gallery and documentation for Vize.
//!
//! ## Name Origin
//!
//! **Musea** (plural of museum) represents a gallery space where art is
//! displayed and documented. Similarly, `vize_musea` provides a gallery
//! for Vue components, allowing developers to view and interact with
//! components in isolation - similar to Storybook.
//!
//! ## Concepts
//!
//! - **Art**: A component variation/state (replaces "story")
//! - **Art file** (`*.art.vue`): File defining arts
//! - **Gallery**: Display area for arts
//! - **Palette**: Interactive controls panel
//!
//! ## Performance
//!
//! This crate is optimized for high performance:
//! - **Zero-copy parsing**: All strings are borrowed from source
//! - **Arena allocation**: Uses `vize_carton::Bump` for fast allocation
//! - **Minimal allocations**: Only allocates when absolutely necessary
//! - **Fast byte-level parsing**: Uses `memchr` and `memmem` for O(n) search
//!
//! ## Usage
//!
//! ```rust
//! use vize_carton::Bump;
//! use vize_musea::{parse_art, transform_to_csf};
//! use vize_musea::types::ArtParseOptions;
//!
//! let allocator = Bump::new();
//! let source = r#"
//! <art title="Button" component="./Button.vue">
//!   <variant name="Primary" default>
//!     <Button variant="primary">Click me</Button>
//!   </variant>
//!   <variant name="Secondary">
//!     <Button variant="secondary">Click me</Button>
//!   </variant>
//! </art>
//!
//! <script setup lang="ts">
//! import Button from './Button.vue'
//! </script>
//! "#;
//!
//! // Parse Art file with arena allocator
//! let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
//!
//! // Transform to Storybook CSF
//! let csf = transform_to_csf(&art);
//! println!("Generated: {}", csf.filename);
//! ```
//!
//! ## Features
//!
//! - Zero-copy parsing of `*.art.vue` files
//! - Type-safe variant definitions
//! - Storybook CSF 3.0 export
//! - Visual Regression Testing (VRT) support
//! - Interactive props palette

pub mod autogen;
pub mod docs;
pub mod palette;
pub mod parse;
pub mod transform;
pub mod types;
pub mod vrt;

// Re-exports for convenience
pub use parse::parse_art;
pub use transform::{transform_to_csf, transform_to_vue};
pub use types::{
    ArtDescriptor, ArtDescriptorOwned, ArtMetadata, ArtMetadataOwned, ArtParseError,
    ArtParseOptions, ArtParseResult, ArtScriptBlock, ArtScriptBlockOwned, ArtStatus, ArtStyleBlock,
    ArtStyleBlockOwned, ArtVariant, ArtVariantOwned, CsfOutput, SourceLocation, ViewportConfig,
};

// Re-export vize_carton::Bump for convenience
pub use vize_carton::Bump;

/// Start the Musea component gallery server.
///
/// This function starts a development server that serves the component gallery UI.
pub fn serve() {
    todo!("Component gallery server for Vue SFC - implement with Vite plugin")
}

#[cfg(test)]
mod tests {
    use super::{
        parse_art, transform_to_csf, transform_to_vue, ArtDescriptorOwned, ArtParseOptions, Bump,
    };

    #[test]
    fn test_full_workflow() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button" description="A versatile button component" component="./Button.vue" category="atoms" tags="ui,input">
  <variant name="Primary" default>
    <Button variant="primary">Primary Button</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Secondary Button</Button>
  </variant>
  <variant name="With Icon">
    <Button variant="primary" icon="plus">Add Item</Button>
  </variant>
</art>

<script setup lang="ts">
import Button from './Button.vue'
</script>

<style scoped>
.art-container {
  padding: 20px;
}
</style>
"#;

        // Parse with arena allocator
        let art = parse_art(
            &allocator,
            source,
            ArtParseOptions {
                filename: "Button.art.vue".into(),
            },
        )
        .unwrap();

        assert_eq!(art.metadata.title, "Button");
        assert_eq!(
            art.metadata.description,
            Some("A versatile button component")
        );
        assert_eq!(art.metadata.category, Some("atoms"));
        assert_eq!(art.metadata.tags.len(), 2);
        assert_eq!(art.variants.len(), 3);
        assert!(art.script_setup.is_some());
        assert_eq!(art.styles.len(), 1);

        // Transform to CSF
        let csf = transform_to_csf(&art);
        insta::assert_debug_snapshot!(csf);

        // Transform to Vue
        let vue = transform_to_vue(&art);
        insta::assert_debug_snapshot!(vue);
    }

    #[test]
    fn test_default_variant() {
        let allocator = Bump::new();
        let source = r#"
<art title="Test">
  <variant name="First">
    <div>First</div>
  </variant>
  <variant name="Second" default>
    <div>Second</div>
  </variant>
</art>
"#;

        let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
        let default = art.default_variant().unwrap();
        assert_eq!(default.name, "Second");
    }

    #[test]
    fn test_into_owned() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="Primary">
    <Button>Click</Button>
  </variant>
</art>
"#;

        let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
        let owned: ArtDescriptorOwned = art.into_owned();

        assert_eq!(owned.metadata.title, "Button");
        assert_eq!(owned.variants.len(), 1);
    }

    #[test]
    fn test_arena_efficiency() {
        // Test that multiple parses can share an allocator
        let allocator = Bump::new();

        let sources = [
            r#"<art title="A"><variant name="V1"><div>1</div></variant></art>"#,
            r#"<art title="B"><variant name="V2"><div>2</div></variant></art>"#,
            r#"<art title="C"><variant name="V3"><div>3</div></variant></art>"#,
        ];

        for source in sources {
            let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
            assert!(!art.metadata.title.is_empty());
        }

        // All allocations in single arena - efficient memory usage
        // Arena is dropped when allocator goes out of scope
    }
}
