//! # Vize
//!
//! High-performance Vue.js toolchain written in Rust.
//!
//! This crate re-exports all Vize sub-crates for unified documentation.
//!
//! ## Crates
//!
//! - [`carton`] - Shared allocator, string, hash, and utility types
//! - [`relief`] - Vue template AST, errors, and compiler options
//! - [`armature`] - Vue template tokenizer and parser
//! - [`atelier_core`] - Core template compiler infrastructure
//! - [`atelier_dom`] - DOM mode template compiler
//! - [`atelier_vapor`] - Vapor mode template compiler
//! - [`atelier_sfc`] - Single File Component (SFC) parser and compiler
//! - [`glyph`] - Vue SFC formatter
//! - [`patina`] - Vue SFC linter
//! - [`canon`] - Vue-aware type checking and virtual TS generation
//! - [`musea`] - Musea art parsing and documentation core
//! - [`maestro`] - Language Server Protocol (LSP) implementation

/// Shared allocator, string, hash, and utility types.
pub use vize_carton as carton;

/// Vue template AST, errors, and compiler options.
pub use vize_relief as relief;

/// Vue template tokenizer and parser.
pub use vize_armature as armature;

/// Core template compiler infrastructure.
pub use vize_atelier_core as atelier_core;

/// DOM mode template compiler.
pub use vize_atelier_dom as atelier_dom;

/// Vapor mode template compiler.
pub use vize_atelier_vapor as atelier_vapor;

/// Single File Component (SFC) parser and compiler.
pub use vize_atelier_sfc as atelier_sfc;

/// Vue SFC formatter.
#[cfg(feature = "glyph")]
pub use vize_glyph as glyph;

/// Vue SFC linter.
pub use vize_patina as patina;

/// Vue-aware type checking and virtual TS generation.
pub use vize_canon as canon;

/// Musea art parsing and documentation core.
pub use vize_musea as musea;

/// Language Server Protocol (LSP) implementation.
pub use vize_maestro as maestro;
