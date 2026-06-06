//! # vize_armature
//!
//! Armature - The structural parser framework for Vize.
//! Vue template tokenizer and parser implementation.
//!
//! ## Name Origin
//!
//! **Armature** (/ˈɑːrmətʃər/) is the internal skeleton or framework that supports
//! a sculpture during its creation. Similarly, `vize_armature` provides the parsing
//! framework that builds the AST structure defined in `vize_relief`.
//!
//! ## Features
//!
//! - High-performance HTML tokenizer optimized for Vue templates
//! - State machine-based parsing
//! - Full Vue directive and interpolation support
//! - Error recovery and detailed error reporting

pub mod parser;
pub mod tokenizer;

/// Legacy Vue (v0 / v1 / v2) support. Gated behind the `legacy` feature and
/// dropped from the default Vue 3 build; opt-in only.
#[cfg(feature = "legacy")]
pub mod legacy;

pub use parser::*;
pub use tokenizer::*;

// Re-export from vize_relief for convenience
pub use vize_relief::ast::*;
pub use vize_relief::{
    AllocBox, AllocVec, Allocator, CloneIn, CompilerError, CompilerResult, ErrorCode, ParseMode,
    ParserOptions, TextMode, WhitespaceStrategy,
};
