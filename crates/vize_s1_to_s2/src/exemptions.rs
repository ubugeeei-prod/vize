//! The lowering's error kinds that carry no witness yet — exempt from P4-6's
//! witness law **by inventory**, never silently.
//!
//! `vize_davinci::diagnostic` makes an unwitnessed error unrepresentable
//! except through a declared [`Exemption`]. Each static below is one such
//! declaration, counted in `davinci-road/plan/witness-exemptions.tsv` with the
//! number of construction sites that report under it. The inventory only
//! shrinks: a row leaves it when its site reports through
//! `Diagnostic::proven` with a fact chain instead. Every kind here is a
//! structural fact about the authored template, so each is drained by a fact
//! group over S1/S2 that proves it.

use vize_davinci::diagnostic::Exemption;

/// S1 tokenizer errors (`SurfaceError`), reported at their offset.
pub static SURFACE_SYNTAX: Exemption = Exemption::new("vize_s1_to_s2", "surface-syntax");

/// An element whose close tag is an `ElementClose::Missing` hole.
pub static MISSING_END_TAG: Exemption = Exemption::new("vize_s1_to_s2", "missing-end-tag");

/// The structural lowering's rejections, reported through one helper:
/// `v-else`/`v-else-if` without an adjacent `v-if`, a `v-if`/`v-for`/`v-model`
/// without its expression, an invalid `v-for` expression, a custom directive
/// or `v-model` on a `<slot>` outlet, two `v-if` branches with the same key,
/// and node-id exhaustion.
pub static LOWERING: Exemption = Exemption::new("vize_s1_to_s2", "lowering");

/// The `v-slot` grouping pass's rejections: a misplaced `v-slot`, mixed
/// default and named slot usage, a duplicate slot name, and extraneous
/// children beside named slots.
pub static V_SLOT: Exemption = Exemption::new("vize_s1_to_s2", "v-slot");

/// The `v-model` canonicalization pass's rejections: a `v-model` on a
/// `v-for` or `v-slot` scope variable, and an argument on a plain element.
pub static V_MODEL: Exemption = Exemption::new("vize_s1_to_s2", "v-model");
