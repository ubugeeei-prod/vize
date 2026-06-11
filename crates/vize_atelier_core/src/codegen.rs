//! VDom code generation.
//!
//! This module generates JavaScript render function code from the transformed AST.

mod children;
mod context;
mod element;
mod expression;
mod generate;
mod helpers;
mod node;
mod patch_flag;
mod pipeline;
mod props;
mod root;
mod slots;
mod v_for;
mod v_if;

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests;

#[cfg(test)]
use crate::options::CodegenOptions;

pub use context::{CodegenContext, CodegenResult, CodegenResultWithSections, CodegenSections};
pub(crate) use helpers::is_constant_simple_expression;
pub use pipeline::{generate, generate_with_sections};
