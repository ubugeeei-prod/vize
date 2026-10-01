//! Dialect-specific contracts used at the L1→L2 legalization boundary.
//!
//! Capability derivation has one owner in L1. The compact projection preserves
//! the existing lowering layout without importing a legacy product crate.

pub use vize_l1::dialect::LegacyCaps;
