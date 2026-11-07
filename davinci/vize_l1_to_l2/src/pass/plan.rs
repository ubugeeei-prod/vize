//! Artifact-scoped L2 transform plans.
//!
//! The static transform table stays the review unit, but the build path
//! should not pay a diagnostic pass for an op family the lowering did not
//! produce. Feature bits come from the lowering construction sites, not a
//! second L2 walk.
//!
//! # Why the plan is a mask, not a match arm
//!
//! Each selectable pass is declined independently, so reachable plans are
//! subsets of the static transform table. Compile-time assertions keep every
//! plan shape pinned. A declined pass may cost nothing observable: its op
//! family was never built or the emitter cannot consume its optional facts.
//!
//! Vue legalization policy owns the implementation under `dialect/vue`; this
//! ordinary child module preserves the existing API and test identities.

include!("../dialect/vue/pass/plan.rs");
