//! Vue template AST node types.
//!
//! This module defines the AST (Abstract Syntax Tree) for Vue templates.
//! All AST nodes are allocated in a bumpalo arena for efficient memory management
//! and zero-copy transfer to JavaScript.

pub mod codegen;
pub mod control_flow;
pub mod core;
pub mod elements;
pub mod expressions;
pub mod nodes;

#[cfg(test)]
mod tests;

pub use codegen::*;
pub use control_flow::*;
pub use core::*;
pub use elements::*;
pub use expressions::*;
pub use nodes::*;
