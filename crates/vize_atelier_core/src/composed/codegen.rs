// DO NOT EDIT. Generated from crates/vize_atelier_core/src/codegen.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
//! VDOM code generation.
//!
//! This module generates JavaScript render function code from the transformed AST.

/// Push the setup binding a component tag resolves to, as
/// [`CodegenContext::push_component_binding_tag`] does. A composed map also
/// anchors the read at the authored tag.
macro_rules! push_component_binding {
    ($ctx:expr, $el:expr) => {
        render_map_copy!(
            plain { $ctx.push_component_binding_tag($el.tag) }
            composed { $ctx.push_composed_component_binding($el) }
        )
    };
}

mod children;
mod component_binding;
mod context;
pub mod document;
mod element;
pub(crate) use element::helpers::is_dynamic_component;
mod emit;
mod entry;
mod expression;
mod generate;
render_map_copy! {
    plain {
        #[cfg(test)]
        mod for_alias_spans;
    }
    composed {
        mod for_alias_spans;
        pub use for_alias_spans::locate_for_aliases;
    }
}
mod helpers;
mod node;
mod patch_flag;
mod props;
pub mod rewrite_spans;
mod root;
mod slots;
pub mod source_map;
pub mod source_map_anchor;
mod v_for;
mod v_if;

#[cfg(test)]
#[expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]
mod tests;

#[cfg(test)]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]
mod span_tests;

#[cfg(test)]
mod component_name_literal_tests;

#[cfg(test)]
use crate::composed::options::CodegenOptions;

pub use context::{CodegenContext, CodegenResult, CodegenResultWithSections, CodegenSections};
pub use expression::convert_line_comments_to_block;
// `pub` (P2-9 series 6): the Davinci differential comparator drives the
// shipped constant classifier from test space to detect where the L2
// lane's deliberately weaker const rule diverges (`consts_templates`),
// so the oracle is this function itself rather than a drift-prone copy.
pub use helpers::is_constant_simple_expression;
// Shared with the dialect-gated Vue 2 filter transform, which builds the same
// `_filter_<name>` asset id the codegen preamble declares.
pub use entry::{
    generate, generate_with_experimental_options, generate_with_merge_props,
    generate_with_sections, generate_with_sections_and_experimental_options,
    generate_with_sections_and_experimental_options_and_slot_scope,
    generate_with_sections_and_experimental_options_slot_scope_and_environment,
    generate_with_vnode_factory, generate_with_vnode_factory_and_child_values,
    generate_with_vnode_factory_and_merge_props,
};
#[cfg(feature = "legacy")]
pub(crate) use helpers::to_valid_asset_identifier;
pub(crate) use helpers::{escape_js_string, is_valid_js_identifier};
