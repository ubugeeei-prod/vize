//! Inline mode script compilation.
//!
//! This module handles compilation of script setup with inline template mode,
//! where the render function is inlined into the setup function.

mod compiler;
pub(crate) mod helpers;
#[cfg(test)]
mod tests;
pub(crate) mod type_handling;

pub use compiler::compile_script_setup_inline;
pub(crate) use compiler::compile_script_setup_inline_with_context;
