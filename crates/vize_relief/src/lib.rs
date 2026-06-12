//! # vize_relief
//!
//! Relief - The sculptured AST surface for Vize.
//! Vue template Abstract Syntax Tree definitions, errors, and compiler options.
//!
//! ## Name Origin
//!
//! **Relief** (/rɪˈliːf/) is a sculptural technique where figures project from a flat
//! background, creating depth and dimension. Like how relief carving reveals forms
//! from a surface, `vize_relief` defines the structural forms (AST nodes) that
//! represent Vue template syntax.
//!
//! ## Features
//!
//! - Complete Vue template AST node definitions
//! - Compiler error types and codes
//! - Parser, transform, and codegen options
//! - Arena-allocated nodes for zero-copy JavaScript interop
//! - Serialization support with serde

pub mod errors;
pub mod options;
mod relief;

pub use errors::*;
pub use options::*;
pub use relief::*;

/// Re-export allocator types for convenience
pub use vize_carton::{Allocator, Box as AllocBox, CloneIn, Vec as AllocVec};
