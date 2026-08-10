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

#[cfg(feature = "wasm")]
mod analyze;
mod ast;
mod compiler;
#[cfg(feature = "wasm")]
mod cross_file;
#[cfg(feature = "wasm")]
mod cross_file_complexity;
mod experimentals;
#[cfg(feature = "glyph")]
mod format;
#[cfg(feature = "wasm")]
mod inspector;
#[cfg(feature = "wasm")]
mod jsx;
#[cfg(feature = "wasm")]
mod lint;
#[cfg(feature = "wasm")]
mod musea;
mod options;
#[cfg(feature = "wasm")]
mod reactivity_overlay;
mod serde;
mod sfc_types;
#[cfg(feature = "wasm")]
mod source_offsets;

#[cfg(test)]
mod tests;

// Re-export type checking bindings from separate module
#[cfg(feature = "wasm")]
#[path = "wasm_typecheck.rs"]
mod wasm_typecheck;

// Re-export all WASM bindings
#[cfg(feature = "wasm")]
pub use analyze::*;
pub use compiler::*;
#[cfg(feature = "wasm")]
pub use cross_file::*;
#[cfg(feature = "glyph")]
pub use format::*;
#[cfg(feature = "wasm")]
pub use inspector::*;
#[cfg(feature = "wasm")]
pub use jsx::*;
#[cfg(feature = "wasm")]
pub use lint::*;
#[cfg(feature = "wasm")]
pub use musea::*;
pub use sfc_types::*;
#[cfg(feature = "wasm")]
pub use wasm_typecheck::*;

// Re-export shared helpers so sibling submodules can reach them via `super::`.
#[cfg(feature = "wasm")]
pub(crate) use serde::{to_js_value, to_json_js_value, utf8_byte_to_utf16_offset};
