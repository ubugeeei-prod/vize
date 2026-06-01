//! Opinionated rule bundles.
//!
//! These rules go beyond the general happy path and enforce stronger stylistic,
//! structural, and framework-specific conventions.

pub(crate) mod a11y;
pub(crate) mod html;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod type_aware;
pub(crate) mod vapor;
pub(crate) mod vue;

use crate::rule::RuleRegistry;

pub(crate) fn register(registry: &mut RuleRegistry) {
    vue::register(registry);
    vapor::register(registry);
    a11y::register(registry);
    html::register(registry);
    #[cfg(not(target_arch = "wasm32"))]
    type_aware::register(registry);
}

pub(crate) fn register_nuxt(registry: &mut RuleRegistry) {
    vue::register_nuxt(registry);
    vapor::register(registry);
    a11y::register(registry);
    html::register(registry);
    #[cfg(not(target_arch = "wasm32"))]
    type_aware::register(registry);
}
