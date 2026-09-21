//! SFC parsing implementation.
//!
//! Zero-copy design with byte-level operations for maximum performance.
//! Uses Cow<str> to avoid string allocations during parsing.

mod block;
mod parse_sfc;
mod template_boundary;

#[cfg(test)]
mod block_location_tests;
#[cfg(test)]
mod regex_recovery_tests;
#[cfg(test)]
mod skip_css_vars_tests;
#[cfg(test)]
mod tests;

pub use parse_sfc::{parse_sfc, parse_sfc_without_css_vars};
