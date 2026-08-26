//! Relief IR node types.
//!
//! This module defines the lowered relief IR that both template and JSX lowering
//! target. All nodes are allocated in the per-compile arena
//! (`vize_s0::Allocator`, a wrapper over `oxc_allocator`) and hold their
//! strings as `&'a str` slices into that arena or into the source text, so a
//! tree costs one pool and no per-node heap allocation.

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
