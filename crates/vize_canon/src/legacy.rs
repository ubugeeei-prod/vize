//! Legacy Vue (v0.10 / v0.11 / v1 / v2) type-checking surface.
//!
//! This module is the consolidation point for pre-Vue-3 ("legacy") type
//! checking and virtual-TS generation in `vize_canon`. It is gated behind the
//! `legacy` cargo feature (which also turns on `vize_armature/legacy`) and is
//! **not** compiled into the default Vue 3 (`vize`) binary: legacy support is
//! strictly opt-in.
//!
//! The version model is shared from [`vize_armature::legacy`] so the parser,
//! type checker, and editor layers all agree on which legacy line is in play,
//! and dialects are resolved once per file into a
//! [`LegacyDialectCapabilities`] set rather than re-matched in hot paths.

pub use vize_armature::legacy::{DirectiveArgStyle, LegacyDialectCapabilities, LegacyVueVersion};
