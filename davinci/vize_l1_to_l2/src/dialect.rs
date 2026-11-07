//! Dialect-specific contracts used at the L1→L2 legalization boundary.
//!
//! Capability derivation has one owner in L1. Legalization consumes that same
//! descriptor without importing a legacy product crate or maintaining a
//! second, drift-prone projection.

pub mod legalize;

// These disabled ordinary declarations keep every physically owned policy
// body in the standard rustfmt module graph without changing runtime identity.
#[cfg(any())]
mod exports;
#[cfg(any())]
mod vue;
#[cfg(any())]
mod vue2;
#[cfg(any())]
mod vue3;

pub use vize_l1::dialect::LegacyCaps;
