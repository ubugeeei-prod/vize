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
#[derive(Debug, Clone)]
pub struct PropsDestructureBinding {
    /// Local variable name
    pub local: String,
    /// Default value expression (source text)
    pub default: Option<String>,
}

/// Props destructure bindings data
#[derive(Debug, Clone, Default)]
pub struct PropsDestructuredBindings {
    /// Map of prop key -> binding info
    pub bindings: FxHashMap<String, PropsDestructureBinding>,
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
