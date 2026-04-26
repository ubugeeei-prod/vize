//! NAPI bindings for Vue compiler.
//!
//! This module is organized into:
//! - `template`: Template compilation (compile, compileVapor, parseTemplate)
//! - `sfc`: SFC parsing, compilation, and batch processing
//! - `art`: Art file parsing, CSF transform, docs, palette, and autogen
//! - `format`: Vue SFC formatting
//! - `lint`: Vue SFC linting

mod art;
mod format;
mod lint;
mod sfc;
mod template;

#[path = "../napi_typecheck.rs"]
mod napi_typecheck;
pub use napi_typecheck::*;

pub use art::*;
pub use format::*;
pub use lint::*;
pub use sfc::*;
pub use template::*;
