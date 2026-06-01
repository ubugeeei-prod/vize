//! SFC parsing implementation.
//!
//! Zero-copy design with byte-level operations for maximum performance.
//! Uses Cow<str> to avoid string allocations during parsing.

mod block;
mod parse_sfc;

#[cfg(test)]
mod tests;

pub use parse_sfc::parse_sfc;
