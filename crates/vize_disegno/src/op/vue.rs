//! The `vue.*` dialect family: genuinely Vue-specific ops that ride along
//! the neutral core instead of shaping it.
//!
//! One op lives here unconditionally, and that is the rule, not a gap: a
//! dialect op lands with the transform that needs it (P2-9), never
//! speculatively. The fairness litmus test (P2-16) is what keeps this
//! family honest - a lint rule written against `ui.*` must run unchanged
//! on SFC and JSX, so anything only Vue understands belongs here. The
//! legacy-port installment (P2-9 series 7) added [`VueFilterOp`] behind
//! the `_legacy` feature: a Vue 2 filter chain has no modern equivalent
//! (unlike `.sync` and `slot-scope`, whose meanings *are* their Vue 3
//! desugar targets - the record's measured dialect-op test), so it is
//! the one legacy surface that earns vocabulary.

use vize_carton::{Span, Vec};

use super::DynamicName;
use crate::expr::ExprRef;

/// `vue.directive` - a Vue custom directive (`v-pin:top.lazy="value"`),
/// carried through S2 for a consumer that understands it.
///
/// Built-in directives never appear here: they normalize into `ui.*` ops
/// at lowering (`v-if` into [`super::IfOp`], `v-model` into
/// [`super::ModelOp`], ...); only user-defined directives survive as
/// dialect ops, exactly as the shipped pipeline emits runtime directive
/// references for them today.
#[derive(Debug)]
pub struct VueDirectiveOp<'a> {
    /// Directive name without the `v-` prefix, a slice of the source.
    pub name: &'a str,
    /// The authored argument (`v-pin:top`, `v-pin:[dir]`), when present.
    pub argument: Option<DynamicName<'a>>,
    /// Modifier names in authored order, without their leading dots.
    pub modifiers: Vec<'a, &'a str>,
    /// The directive's value expression, when authored.
    pub value: Option<ExprRef<'a>>,
    /// The whole directive's source range.
    pub span: Span,
}

/// See [`crate::op`] for the guard rationale.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<VueDirectiveOp<'_>>() == 88);

/// `vue.filter` - a Vue 2 pipe-filter interpolation
/// (`{{ msg | capitalize }}`), the P2-9 series-7 flagship dialect op.
///
/// Only the `_legacy` lowering under a filter-capable dialect constructs
/// one; the default dialect never sees the variant, and a build without
/// the feature does not compile it at all. The expression is pessimally
/// opaque ([`crate::expr::OpaqueReason::LegacyFilter`]: the authored text
/// is not a JS expression under the dialect - `|` is a filter pipe, not
/// bitwise-or - so no AST may be retained); the split structure (base +
/// segments) rides beside the tree exactly as the Compound producer's
/// parts do (`vize_ricalco::lower::Lowered::filters`), validated and
/// published by the legacy pass.
#[cfg(feature = "_legacy")]
#[derive(Debug)]
pub struct VueFilterOp<'a> {
    /// The whole filtered expression, byte-verbatim (trimmed), under the
    /// pessimal opaque laws.
    pub expression: ExprRef<'a>,
    /// The whole interpolation's source range (delimiters included).
    pub span: Span,
}

/// See [`crate::op`] for the guard rationale.
#[cfg(all(feature = "_legacy", target_pointer_width = "64"))]
const _: () = assert!(core::mem::size_of::<VueFilterOp<'_>>() == 24);
