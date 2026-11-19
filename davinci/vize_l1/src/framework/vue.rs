//! Vue's L1 syntax hooks: the one Vue owner (#6841,
//! `docs/davinci/decisions/2026-11-17-input-neutral-levels.md`).
//!
//! [`quirks`] is the shared capability model each file resolves once, and
//! [`feature`] the composable syntax features; [`version`] holds one table and
//! the version-only surfaces per major version. Compatibility parser adapters
//! and L1→L2 legalization consume the same facts. [`directive`] decomposes
//! directive attribute names, [`profile`] is the SFC template profile and
//! [`style_bind`] reads CSS `v-bind()`.

pub mod directive;
pub(crate) mod feature;
pub mod profile;
pub(crate) mod quirks;
pub mod style_bind;
pub(crate) mod surface;
pub mod version;

pub use profile::VueTemplate;
pub use quirks::{DirectiveArgStyle, LegacyDialectCapabilities, LegacyVueVersion};

/// Backwards-compatible name for the shared Vue template capability set.
///
/// L1 parsing and L1→L2 legalization intentionally consume the same value;
/// keeping this alias avoids a second projection that can drift as dialect
/// surfaces are added.
pub type LegacyCaps = LegacyDialectCapabilities;
