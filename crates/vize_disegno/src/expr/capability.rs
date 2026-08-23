//! The expression capability contract: how a consumer asks questions
//! about an [`super::ExprRef`], and how the answering code is resolved.
//!
//! # Resolution is per file, never per node (guardrail 1)
//!
//! A file compiles under exactly one expression dialect, so a consumer
//! binds one [`ExprDialect`] implementation **once, at pipeline entry,
//! as a generic parameter** - the P2-3 observer shape: static dispatch,
//! monomorphized walks, and the no-dialect-question path compiles to no
//! check at all. No `dyn ExprDialect` is offered, on purpose; the one
//! designated `dyn` seam of the architecture is S4 emitters, and this is
//! not it.
//!
//! # Who answers what
//!
//! - [`ExprRef::Js`](super::ExprRef::Js): the **core** answers, from the
//!   retained AST. Those answers land with their consumers (the phase-4
//!   projection and lint re-bases); this trait is the seam they share.
//! - [`ExprRef::Opaque`](super::ExprRef::Opaque): the answers are fixed
//!   by the pessimal laws ([`super::opaque`]) and an implementation
//!   **must not** improve on them - may-read-anything, may-do-anything,
//!   never constant, equal to nothing, emit verbatim or refuse.
//! - [`ExprRef::Foreign`](super::ExprRef::Foreign): the dialect answers.
//!   No implementation ships until phase 6 (charter #28).
//! - [`ExprRef::Filter`](super::ExprRef::Filter): Vue 2 pipe filters.
//!   Until the S2 legacy pass legalizes the chain, treat the payload as
//!   non-constant authored text (never a JS parse of the `|`).
//!
//! # The P1-8 note (issue #4365)
//!
//! [`ExprDialect::enumerate_bindings`] is **where the single
//! identifier-extraction implementation would live**: the byte scanner
//! (`identifiers/fast.rs`) and the AST walk (`identifiers/slow.rs`)
//! disagree on 3.85% of corpus extractions, the waiver decision is the
//! maintainer's (`davinci-road/plan/scanner-parity-report.md`), and this
//! trait deliberately encodes **neither** resolution - it names the
//! duty's future home and nothing else. Phase 2 does not unblock P1-8.

use vize_carton::Span;

use super::ExprRef;

/// The per-dialect expression capability contract - the four duties every
/// consumer surface needs, behind one statically-resolved seam.
///
/// See the module docs for the resolution rule (per file, generic, no
/// `dyn`), the per-variant answering table, and the pessimal-law bound on
/// [`ExprRef::Opaque`](super::ExprRef::Opaque) answers. Signatures are
/// deliberately minimal - the first implementor (the core's JS dialect,
/// phase 4; a foreign dialect, phase 6) refines them with its consumer,
/// never speculatively.
pub trait ExprDialect {
    /// Enumerate the bindings `expr` references, in source order.
    ///
    /// Mandatory behavior for an opaque expression: visit nothing here
    /// and answer `false` from [`ExprDialect::bindings_are_exact`], so
    /// the consumer is forced onto the pessimal path (law 1) - an
    /// implementation must not "best-effort scan" opaque text.
    fn enumerate_bindings(&self, expr: ExprRef<'_>, each: &mut dyn FnMut(&str));

    /// Whether [`ExprDialect::enumerate_bindings`] enumerated *exactly*
    /// the referenced bindings for `expr`.
    ///
    /// `false` means the enumeration is a lower bound and the consumer
    /// must assume every reachable binding is referenced - the mandatory
    /// answer for an opaque expression (pessimal law 1).
    fn bindings_are_exact(&self, expr: ExprRef<'_>) -> bool;

    /// Classify const-ness: may the expression be folded, hoisted, or
    /// cached? Mandatory answer for an opaque expression: `false`
    /// (pessimal law 3, [`super::OpaqueExpr::is_constant`]).
    fn is_constant(&self, expr: ExprRef<'_>) -> bool;

    /// Map a range inside the expression's text (relative to
    /// [`ExprRef::source`](super::ExprRef::source)) to the authored range
    /// in the compiled file.
    fn map_span(&self, expr: ExprRef<'_>, inner: Span) -> Span;

    /// Emit the expression for the bound target, or refuse.
    ///
    /// Mandatory behavior for an opaque expression: write
    /// [`ExprRef::source`](super::ExprRef::source) byte-verbatim or
    /// return `Err` (pessimal law 5) - never a "fixed up" spelling.
    fn emit(
        &self,
        expr: ExprRef<'_>,
        out: &mut dyn core::fmt::Write,
    ) -> Result<(), core::fmt::Error>;
}
