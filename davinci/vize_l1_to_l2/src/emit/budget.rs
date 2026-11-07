//! Vue-aware emission orchestration with the existing module identity.
//!
//! Physical policy ownership lives under `dialect/vue`; child declarations
//! remain here so every existing private and public path stays unchanged.

include!("../dialect/vue/emit/budget.rs");

mod preflight;
mod slot_scope;
