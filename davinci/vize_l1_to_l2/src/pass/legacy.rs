//! Compatibility exports for the shared Vue dialect legalizer.
//!
//! Policy is owned by the dialect module; the pass table, walks and existing
//! public entry points retain their exact behavior.

pub use crate::dialect::legalize::{
    DESC, LEGACY, LEGACY_PASSES, LegacyFacts, NAME, pipeline_for, run,
};
