//! Compatibility exports for the shared Vue dialect legalizer.
//!
//! Policy is owned by the dialect module; the pass table, walks and existing
//! public entry points retain their exact behavior and module identity.

include!("../dialect/vue/pass/legacy.rs");
