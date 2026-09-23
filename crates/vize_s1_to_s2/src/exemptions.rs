//! The lowering's error kinds that carry no witness yet — exempt from P4-6's
//! witness law **by inventory**, never silently.
//!
//! `vize_davinci::diagnostic` makes an unwitnessed error unrepresentable
//! except through a declared [`Exemption`]. Each static below is one such
//! declaration, counted in `docs/davinci/plan/witness-exemptions.tsv` with the
//! number of construction sites that report under it. The inventory only
//! shrinks: a row leaves it when its site reports through
//! `Diagnostic::proven` with a fact chain instead. Every kind here is a
//! structural fact about the authored template, so each is drained by a fact
//! group over S1/S2 that proves it.

use vize_davinci::diagnostic::{Diagnostic, Exemption, Stage};
use vize_s0::Span;

/// S1 tokenizer errors of either surface dialect — the HTML tokenizer's
/// `SurfaceError` and the pug lexer's `PugError` — reported at their offset
/// through [`surface_syntax`].
pub static SURFACE_SYNTAX: Exemption = Exemption::new("vize_s1_to_s2", "surface-syntax");

/// An element whose close tag is an `ElementClose::Missing` hole.
pub static MISSING_END_TAG: Exemption = Exemption::new("vize_s1_to_s2", "missing-end-tag");

/// The structural lowering's rejections, reported through one helper:
/// `v-else`/`v-else-if` without an adjacent `v-if`, a `v-if`/`v-for`/`v-model`
/// without its expression, an invalid `v-for` expression, a custom directive
/// or `v-model` on a `<slot>` outlet, two `v-if` branches with the same key,
/// and node-id exhaustion; and the pug desugaring's refusals of constructs
/// that need a JavaScript engine or other files at build time (mixins,
/// includes, conditionals, iteration, code, interpolation, non-constant
/// attributes). All go through [`lowering`].
pub static LOWERING: Exemption = Exemption::new("vize_s1_to_s2", "lowering");

/// The `v-slot` grouping pass's rejections: a misplaced `v-slot`, mixed
/// default and named slot usage, a duplicate slot name, and extraneous
/// children beside named slots.
pub static V_SLOT: Exemption = Exemption::new("vize_s1_to_s2", "v-slot");

/// The `v-model` canonicalization pass's rejections: a `v-model` on a
/// `v-for` or `v-slot` scope variable, and an argument on a plain element.
pub static V_MODEL: Exemption = Exemption::new("vize_s1_to_s2", "v-model");

/// The one construction site of [`SURFACE_SYNTAX`].
pub(crate) fn surface_syntax(span: Span, message: &str) -> Diagnostic {
    Diagnostic::legacy_error(&SURFACE_SYNTAX, Stage::Surface, span, message)
}

/// The one construction site of [`LOWERING`].
pub(crate) fn lowering(span: Span, message: &str) -> Diagnostic {
    Diagnostic::legacy_error(&LOWERING, Stage::Semantic, span, message)
}
