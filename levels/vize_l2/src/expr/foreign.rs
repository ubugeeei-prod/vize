//! [`ForeignExpr`] - the non-JS expression dialect payload.
//!
//! Retained neutral-IR payload for opaque expression dialects and historical
//! folio codecs. No non-JavaScript authoring dialect is currently shipped.
//! Source spans remain exact without invoking the JavaScript parser.

use vize_l0::{Span, Vec};

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
    /// The dialect id (a non-empty lowercase word;
    /// the producer owns the vocabulary).
    pub dialect: &'a str,
    /// The exact authored text, opaque to the core.
    pub source: &'a str,
    /// The authored range of `source` in the compiled file.
    pub span: Span,
    /// The dialect's side tables, riding as opaque fact rows. The core
    /// never reads them; the producing dialect owns their
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
