//! # vize_relief
//!
//! Relief - The sculptured AST surface for Vize.
//! Vue template Abstract Syntax Tree definitions, errors, and compiler options.
//! Relief answers what source syntax was written and where. It does not resolve
//! symbol identity, scopes, dependencies, or control flow; Croquis owns those
//! derived semantic relationships.
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
mod product;
mod relief;
mod snapshot;

pub use errors::*;
pub use options::*;
pub use product::{ReliefProduct, VueDialectInput};
pub use relief::*;
pub use snapshot::*;

/// Re-export allocator types for convenience
pub use vize_carton::{Allocator, Box as AllocBox, CloneIn, Vec as AllocVec};
