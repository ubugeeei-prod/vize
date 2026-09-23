//! Corsa integration for collecting native TypeScript diagnostics.
//!
//! This module generates virtual TypeScript from Vue SFCs and uses the Corsa
//! LSP bridge to collect type-checking diagnostics.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
)]

#[cfg(test)]
mod art_variant_tests;
pub(in crate::ide) mod collect;
mod collect_variant;
mod collect_virtual;
mod log_preview;
pub(in crate::ide) use collect_virtual::corsa_diagnostic_code;
mod message;
pub(in crate::ide) use message::rewrite_corsa_message;
mod virtual_ts;
pub(in crate::ide) use virtual_ts::semantic_links_after_import_rewrite;
mod virtual_ts_art;
mod virtual_ts_art_bindings;
mod virtual_ts_inline_art;

#[cfg(test)]
mod relative_import_tests;
#[cfg(test)]
mod semantic_link_tests;
#[cfg(test)]
mod tests;
