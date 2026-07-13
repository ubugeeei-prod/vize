//! Function mode script compilation.
//!
//! This module handles compilation of script setup in function mode,
//! where the setup function returns bindings for use by a separate render function.
//! The compilation is split into:
//! - `compiler`: the main `compile_script_setup` entry point
//! - `helpers`: utility functions (backtick counting, TS detection, reserved words, top-level await)
//! - `imports`: import deduplication logic

mod compiler;
pub(crate) mod helpers;
pub(crate) mod imports;
mod prop_defaults;
mod source;

pub(crate) use compiler::compile_script_setup_from_source;
pub use helpers::contains_top_level_await;
pub use imports::dedupe_imports;
pub use source::compile_script_setup;
