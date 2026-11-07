//! A const native dialect pattern table consumed during owner construction.
//!
//! Vue 3 owns the implementation under `dialect/vue3`; the ordinary child
//! declaration remains here to preserve its existing identity.

mod vue3;

#[cfg(test)]
mod tests;

include!("../dialect/vue3/native/pattern.rs");
