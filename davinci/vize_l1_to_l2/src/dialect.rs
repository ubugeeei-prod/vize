//! Dialect-specific contracts used at the L1→L2 legalization boundary.
//!
//! Capability derivation has one owner in L1. Legalization consumes that same
//! descriptor without importing a legacy product crate or maintaining a
//! second, drift-prone projection.

pub mod legalize;

pub use vize_l1::dialect::LegacyCaps;
