// Language-owned implementation included by `expr::foreign` so the existing
// public type identity remains stable.

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
    /// vocabulary **and their dump spelling** - until it exists, the
    /// dump payload is dialect + source + span only (the P2-5b
    /// contract's list), so a value carrying facts does not round-trip
    /// through a dump yet. That is part of what "type-only" means.
    pub facts: Vec<'a, ForeignFact<'a>>,
}

impl<'a> ForeignExpr<'a> {
    /// Stable target qualifier for this residual language payload.
    #[must_use]
    pub const fn qualifier(&self) -> &'a str {
        self.dialect
    }
}

/// See [`crate::op`] for both guard rationales.
const _: () = assert!(!core::mem::needs_drop::<ForeignExpr<'static>>());
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<ForeignFact<'_>>() == 32);
    assert!(core::mem::size_of::<ForeignExpr<'_>>() == 64);
};
