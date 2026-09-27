//! Diagnostics: the one channel every renderer reads.

pub mod diagnostic;
pub mod witness;

pub use diagnostic::{Diagnostic, Severity};
pub use witness::{WitnessChain, WitnessLink};
