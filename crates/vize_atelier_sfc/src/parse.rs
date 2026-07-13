//! SFC parsing implementation.
//!
//! Zero-copy design with byte-level operations for maximum performance.
//! Uses Cow<str> to avoid string allocations during parsing.

mod block;
mod parse_sfc;
mod structure;
mod template_boundary;

#[cfg(test)]
mod tests;

pub use parse_sfc::parse_sfc;
#[cfg(test)]
pub(crate) use parse_sfc::parse_sfc_call_count;
pub use structure::SfcSourceStructure;
pub(crate) use structure::scan_sfc_structure;
