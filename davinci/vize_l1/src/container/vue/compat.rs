//! The compatibility SFC descriptor: zero-copy blocks projected from the
//! lossless container walk.
//!
//! [`parse_sfc`] reads every block boundary, attribute and source location
//! from [`boundary::split_blocks`](super::boundary::split_blocks) and applies
//! only the shipped descriptor policy (one template, one `<script>`, one
//! `<script setup>`, styles, custom blocks, `v-bind()` CSS variables). The
//! native descriptor observation ([`Vue::observe_descriptor`](super::Vue))
//! stays separate: it refuses what it cannot admit, while this descriptor keeps
//! every block, including custom ones.

mod parse;
mod types;
#[doc(hidden)]
pub mod v_bind;

pub use parse::{parse_sfc, parse_sfc_without_css_vars};
pub use types::{
    BlockLocation, PadOption, SfcCustomBlock, SfcDescriptor, SfcError, SfcParseOptions,
    SfcScriptBlock, SfcStyleBlock, SfcTemplateBlock,
};
