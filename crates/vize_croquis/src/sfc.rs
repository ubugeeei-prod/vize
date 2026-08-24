//! Parse-only Vue Single File Component support.
//!
//! This crate exposes the zero-copy SFC descriptor parser without depending on
//! the DOM, SSR, Vapor, script-transform, code-generation, or CSS-engine
//! portions of [`vize_atelier_sfc`](https://docs.rs/vize_atelier_sfc).

#![cfg_attr(
    test,
    allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )
)]

mod css_transform;
mod parse;
mod types;

pub use parse::parse_sfc;
pub use types::{
    BindingMetadata, BindingType, BlockLocation, PadOption, SfcCustomBlock, SfcDescriptor,
    SfcError, SfcParseOptions, SfcScriptBlock, SfcStyleBlock, SfcTemplateBlock,
};

#[doc(hidden)]
pub mod __internal {
    pub use super::css_transform::{
        extract_and_transform_v_bind, extract_and_transform_v_bind_with_scope, find_matching_paren,
        prod_scoped_v_bind_name, scoped_v_bind_name,
    };
}

pub(crate) fn extract_css_vars(css: &str) -> Vec<vize_carton::String> {
    let bump = vize_carton::pool::acquire();
    css_transform::extract_and_transform_v_bind(&bump, css).1
}
