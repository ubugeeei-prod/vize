//! Props Palette - Interactive controls generation for Art components.
//!
//! This module generates interactive control configurations from Art variants,
//! enabling users to manipulate component props in real-time.
//!
//! # Example
//!
//! ```rust
//! use vize_carton::Bump;
//! use vize_musea::{parse_art, ArtParseOptions};
//! use vize_musea::palette::{generate_palette, PaletteOptions};
//!
//! let allocator = Bump::new();
//! let source = r#"
//! <art title="Button">
//!   <variant name="Primary" args='{"variant":"primary","size":"md","disabled":false}'>
//!     <Button :variant="variant" :size="size" :disabled="disabled">Click</Button>
//!   </variant>
//! </art>
//! "#;
//!
//! let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
//! let output = generate_palette(&art, &PaletteOptions::default());
//! println!("{:?}", output.palette.controls);
//! ```

mod codegen;
mod inference;
mod types;

pub use codegen::generate_palette;
pub use inference::infer_control_type;
pub use types::*;
