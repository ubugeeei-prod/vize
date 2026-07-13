//! # Vize
//!
//! High-performance Vue.js toolchain written in Rust.
//!
//! This crate re-exports the public Vize tool and representation crates used by
//! CLI consumers.
//!
//! ## Crates
//!
//! - [`carton`] - Shared allocator, string, hash, and utility types
//! - [`relief`] - Vue template AST, errors, and compiler options
//! - [`armature`] - Vue template tokenizer and parser
//! - [`atelier_core`] - Legacy template transform/codegen compatibility lane
//! - [`atelier_dom`] - DOM mode template compiler
//! - [`atelier_ssr`] - SSR render backend
//! - [`atelier_vapor`] - Vapor mode template compiler
//! - [`atelier_template`] - Independent raw Vue-template frontend
//! - [`atelier_sfc`] - Single File Component (SFC) parser and compiler
//! - [`atelier_jsx`] - JSX/TSX graph-native frontend
//! - [`rendu`] - Frontend-neutral render HIR
//! - [`flow`] - Optional single-file control/data/effect graph
//! - [`module`] - JavaScript/TypeScript module facts and CFG
//! - [`croquis`] - Frontend-neutral semantic facts
//! - [`croquis_cf`] - Opt-in cross-file semantic aggregation
//! - [`glyph`] - Vue SFC formatter
//! - [`patina`] - Document linter for SFC, template, JSX/TSX, and JS/TS sources
//! - [`canon`] - Vue-aware type checking and virtual TS generation
//! - [`musea`] - Musea art parsing and documentation core
//! - [`maestro`] - Language Server Protocol (LSP) implementation

mod commands;
mod config;
mod profile_support;

/// Typed artifact-graph assembly for Vize tools.
pub mod artifact_graph;

/// Shared native CLI entrypoint.
pub mod cli;

/// Shared allocator, string, hash, and utility types.
pub use vize_carton as carton;

/// Vue template AST, errors, and compiler options.
pub use vize_relief as relief;

/// Vue template tokenizer and parser.
pub use vize_armature as armature;

/// Legacy Vue-template transform/codegen compatibility lane.
pub use vize_atelier_core as atelier_core;

/// DOM mode template compiler.
pub use vize_atelier_dom as atelier_dom;

/// SSR backend over frontend-neutral render HIR.
pub use vize_atelier_ssr as atelier_ssr;

/// Vapor mode template compiler.
pub use vize_atelier_vapor as atelier_vapor;

/// Independent raw Vue-template frontend.
pub use vize_atelier_template as atelier_template;

/// Single File Component (SFC) parser and compiler.
pub use vize_atelier_sfc as atelier_sfc;

/// JSX/TSX graph-native frontend.
pub use vize_atelier_jsx as atelier_jsx;

/// Frontend-neutral render HIR.
pub use vize_rendu as rendu;

/// Frontend-neutral single-file control/data/effect graph.
pub use vize_flow as flow;

/// Production-neutral JavaScript/TypeScript module facts and CFG projection.
pub use vize_module as module;

/// Frontend-neutral semantic products and facts.
pub use vize_croquis as croquis;

/// Opt-in cross-file semantic aggregation.
pub use vize_croquis_cf as croquis_cf;

/// Vue SFC formatter.
#[cfg(feature = "glyph")]
pub use vize_glyph as glyph;

/// Document linter for SFC, template, JSX/TSX, and JS/TS sources.
pub use vize_patina as patina;

/// Vue-aware type checking and virtual TS generation.
pub use vize_canon as canon;

/// Musea art parsing and documentation core.
pub use vize_musea as musea;

/// Language Server Protocol (LSP) implementation.
#[cfg(feature = "maestro")]
pub use vize_maestro as maestro;
