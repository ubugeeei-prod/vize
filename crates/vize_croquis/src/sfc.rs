//! Vue Single File Component support kept for Croquis consumers.
//!
//! The zero-copy descriptor and its parser live in the L1 container layer
//! ([`vize_l1::container::vue::compat`]); this module re-exports them under
//! their established paths and keeps the compiler's CSS `v-bind()` rewrite.

mod css_transform;

pub use vize_l1::container::vue::compat::{
    BlockLocation, PadOption, SfcCustomBlock, SfcDescriptor, SfcError, SfcParseOptions,
    SfcScriptBlock, SfcStyleBlock, SfcTemplateBlock, parse_sfc, parse_sfc_without_css_vars,
};
pub use vize_relief::options::{BindingMetadata, BindingType};

#[doc(hidden)]
pub mod __internal {
    pub use super::css_transform::{
        CssVBindEdit, checked_v_bind_expression_ranges, extract_and_transform_v_bind,
        extract_and_transform_v_bind_with_context, extract_and_transform_v_bind_with_edits,
        extract_and_transform_v_bind_with_scope, find_matching_paren, prod_scoped_v_bind_name,
        scoped_v_bind_name, v_bind_expression_ranges,
    };
    pub use vize_l1::container::vue::compat::v_bind::extract_v_bind_vars as extract_css_vars;
}
