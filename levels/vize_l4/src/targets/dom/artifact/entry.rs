//! One-shot entry points: parse → lower → L2 transform → emit, and the
//! two-half render module the dual-run compares.
//!
//! Physical policy ownership lives under `dialect/vue`; this ordinary child
//! module preserves the existing API and type identities.

include!("../../../framework/vue/dom/entry.rs");
