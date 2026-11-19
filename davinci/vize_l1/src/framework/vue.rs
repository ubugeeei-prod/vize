//! Vue's L1 syntax hooks. This is the one Vue owner the version policies and
//! the directive syntax join (`docs/davinci/decisions/2026-11-17-input-neutral-levels.md`).

pub mod profile;
pub mod directive;
pub mod version;
pub(crate) mod feature;
pub(crate) mod quirks;
pub(crate) mod surface;

pub use profile::VueTemplate;
