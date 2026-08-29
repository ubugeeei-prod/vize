//! [`ForeignExpr`] - the non-JS expression dialect payload.
//!
//! **Type-only until phase 6** (charter #28, the MoonBit dialect): the
//! type exists so [`super::ExprRef`] is closed and the folio grammar is
//! total, but no dialect implementation ships, no lowering constructs a
//! value, and the [`super::capability::ExprDialect`] contract has no
//! implementor. Phase 6 owns the first dialect and with it the first
//! real values of this type.

use vize_s0::{Span, Vec};

/// One dialect-attached fact row (see [`ForeignExpr::facts`]).
#[derive(Debug, Clone, Copy)]
pub struct ForeignFact<'a> {
    /// The dialect's key for this fact.
    pub key: &'a str,
    /// The fact's value, opaque to the core.
    pub value: &'a str,
}

/// The foreign-dialect payload behind [`super::ExprRef::Foreign`]:
/// dialect id, exact source text, authored span, and the dialect's side
/// tables.
#[derive(Debug)]
pub struct ForeignExpr<'a> {
    /// The dialect id (`"moonbit"`-style: a non-empty lowercase word;
    /// the phase-6 dialect registry owns the vocabulary).
    pub dialect: &'a str,
    /// The exact authored text, opaque to the core.
    pub source: &'a str,
    /// The authored range of `source` in the compiled file.
    pub span: Span,
    /// The dialect's side tables, riding as opaque fact rows. The core
    /// never reads them; the phase-6 dialect contract owns their
    /// vocabulary **and their folio spelling** - until it exists, the
    /// folio payload is dialect + source + span only (the P2-5b
    /// contract's list), so a value carrying facts does not round-trip
    /// through a folio yet. That is part of what "type-only" means.
    pub facts: Vec<'a, ForeignFact<'a>>,
}

/// See [`crate::op`] for both guard rationales.
const _: () = assert!(!core::mem::needs_drop::<ForeignExpr<'static>>());
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<ForeignFact<'_>>() == 32);
    assert!(core::mem::size_of::<ForeignExpr<'_>>() == 64);
};
