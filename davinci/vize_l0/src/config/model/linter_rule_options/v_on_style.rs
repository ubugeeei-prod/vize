use serde::{Deserialize, Serialize};

/// Preferred spelling for Vue `v-on` directives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VOnStyle {
    /// Prefer `@click`.
    Shorthand,
    /// Prefer `v-on:click`.
    Longform,
}
