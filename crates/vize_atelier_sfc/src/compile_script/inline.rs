//! Inline mode script compilation.
//!
//! This module handles compilation of script setup with inline template mode,
//! where the render function is inlined into the setup function.

mod compiler;
#[cfg(test)]
mod const_hoist_tests;
#[cfg(test)]
mod define_model_tests;
pub(crate) mod helpers;
#[cfg(test)]
mod static_enum_tests;
#[cfg(test)]
#[expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]
mod tests;
pub(crate) mod type_handling;

pub use compiler::compile_script_setup_inline;
pub(crate) use compiler::{SetupTrace, compile_script_setup_inline_with_context};
