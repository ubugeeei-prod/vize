//! Vue's per-version template syntax: one module per major version, each
//! composing its capability table from [`super::feature`] and owning the
//! surfaces only that version reads.

pub mod v0;
pub mod v1;
pub mod v2;
pub mod v3;
