//! One normally owned CSS parse beside the original completed selected File.
//!
//! Vue 3 owns the selection policy under `dialect/vue3`; this ordinary child
//! module preserves the existing API and test identity.

#[cfg(test)]
mod tests;

include!("../dialect/vue3/native_file/selected_scoped.rs");
