//! WASM bindings for Vue compiler.
//!
//! FFI boundary code: uses std types for JavaScript interop.
//!
//! The module is split into cohesive submodules:
//! - `serde`: serialization / offset-conversion helpers shared across bindings
//! - `options`: parsing of compiler and CSS options from JS option objects
//! - `sfc_types`: WASM-serializable SFC descriptor/result types and conversions
//! - `compiler`: the `Compiler` class, free-function aliases, and compile pipeline
//! - `analyze`, `cross_file`, `format`, `inspector`, `lint`, `musea`: feature bindings
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

mod analyze;
mod compiler;
mod cross_file;
#[cfg(feature = "glyph")]
mod format;
mod inspector;
mod jsx;
mod lint;
mod musea;
mod options;
mod serde;
mod sfc_types;

#[cfg(test)]
mod tests;

// Re-export type checking bindings from separate module
#[path = "wasm_typecheck.rs"]
mod wasm_typecheck;

// Re-export all WASM bindings
pub use analyze::*;
pub use compiler::*;
pub use cross_file::*;
#[cfg(feature = "glyph")]
pub use format::*;
pub use inspector::*;
pub use jsx::*;
pub use lint::*;
pub use musea::*;
pub use sfc_types::*;
pub use wasm_typecheck::*;

// Re-export shared helpers so sibling submodules can reach them via `super::`.
pub(crate) use serde::{to_js_value, to_json_js_value, utf8_byte_to_utf16_offset};
