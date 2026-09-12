//! VDOM code generation.
//!
//! This module generates JavaScript render function code from the transformed AST.

mod children;
mod component_binding;
mod context;
mod element;
mod emit;
mod entry;
mod expression;
mod generate;
mod helpers;
mod node;
mod patch_flag;
mod props;
mod root;
mod slots;
pub mod source_map;
mod v_for;
mod v_if;

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests;

#[cfg(test)]
use crate::options::CodegenOptions;

pub use context::{CodegenContext, CodegenResult, CodegenResultWithSections, CodegenSections};
// `pub` (P2-9 series 6): the Davinci differential comparator drives the
// shipped constant classifier from test space to detect where the S2
// lane's deliberately weaker const rule diverges (`consts_templates`),
// so the oracle is this function itself rather than a drift-prone copy.
pub use helpers::is_constant_simple_expression;
// Shared with the dialect-gated Vue 2 filter transform, which builds the same
// `_filter_<name>` asset id the codegen preamble declares.
pub use entry::{
    generate, generate_with_experimental_options, generate_with_merge_props,
    generate_with_sections, generate_with_sections_and_experimental_options,
    generate_with_vnode_factory, generate_with_vnode_factory_and_merge_props,
};
#[cfg(feature = "legacy")]
pub(crate) use helpers::to_valid_asset_identifier;
