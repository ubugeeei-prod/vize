//! defineProps destructure handling.
//!
//! Handles the props destructure pattern: `const { prop1, prop2 = default } = defineProps(...)`
//!
//! This module follows Vue.js core's definePropsDestructure.ts implementation.
//! Uses OXC for AST-based analysis and transformation.

mod collector;
pub(crate) mod helpers;
mod process;
#[cfg(test)]
mod tests;
mod transform;

use vize_carton::{FxHashMap, String};

/// Props destructure binding info
#[derive(Debug, Clone, Default)]
pub struct PropsDestructureBinding {
    /// Local variable name
    pub local: String,
    /// Default value expression (source text)
    pub default: Option<String>,
    /// Whether the runtime default must be wrapped in a `() => (...)` factory
    /// (non-literal, non-function, non-identifier expression). Mirrors Vue's
    /// `needFactoryWrap`.
    pub(crate) default_needs_factory: bool,
    /// Whether the runtime default is a function or bare identifier, in which
    /// case Vue emits `__skip_<key>: true` and does NOT factory-wrap. Mirrors
    /// Vue's `needSkipFactory`.
    pub(crate) default_skip_factory: bool,
}

/// Props destructure bindings data
#[derive(Debug, Clone, Default)]
pub struct PropsDestructuredBindings {
    /// Map of prop key -> binding info
    pub bindings: FxHashMap<String, PropsDestructureBinding>,
    /// Prop keys in source declaration order (matches the iteration order Vue
    /// uses when generating `mergeDefaults` entries).
    pub(crate) keys: Vec<String>,
    /// Rest spread identifier (if any)
    pub rest_id: Option<String>,
}

impl PropsDestructuredBindings {
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty() && self.rest_id.is_none()
    }
}

pub use helpers::gen_props_access_exp;
pub use process::process_props_destructure;
pub use transform::transform_destructured_props;
