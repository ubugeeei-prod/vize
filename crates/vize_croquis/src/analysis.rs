//! Compatibility re-exports for semantic summary types.
//!
//! New code should prefer [`crate::croquis`]. This module remains public so
//! existing downstream imports such as `vize_croquis::analysis::Croquis` keep
//! working.

pub use crate::croquis::*;
