use serde::{Deserialize, Serialize};

/// Preferred spelling for Vue `v-bind` directives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VBindStyle {
    /// Prefer `:class`.
    Shorthand,
    /// Prefer `v-bind:class`.
    Longform,
}
