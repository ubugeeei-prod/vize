//! Compatibility exports for legacy Vue parser consumers.
//!
//! The shared capability tables are owned by L1's per-version dialect modules.
//! This adapter remains gated by Armature's `legacy` feature.

pub use vize_l1::dialect::{DirectiveArgStyle, LegacyDialectCapabilities, LegacyVueVersion};
