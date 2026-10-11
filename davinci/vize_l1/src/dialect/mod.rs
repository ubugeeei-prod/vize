//! Version-specific syntax policy resolved outside generic surface-tree code.
//!
//! Vue capabilities have one shared owner. Compatibility parser adapters and
//! L1→L2 legalization consume the same facts without a reverse legacy edge.
//! Consolidating petite, quirks, markup/language descriptors and residual
//! dialect operations remains tracked in #6841.

mod template;
mod vue;
pub mod vue0;
pub mod vue1;
pub mod vue2;
pub mod vue3;

pub use template::LegacyCaps;
pub use vue::{DirectiveArgStyle, LegacyDialectCapabilities, LegacyVueVersion};
