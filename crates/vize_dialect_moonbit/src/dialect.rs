//! The MoonBit answers to the S2 expression capability contract.
//!
//! [`MoonBitDialect`] is the first implementor of
//! [`vize_s2::expr::capability::ExprDialect`] (charter #28: the trait had
//! none until phase 6). It answers for `ExprRef::Foreign` payloads of
//! dialect [`crate::sfc::DIALECT`]; every other variant gets the pessimal
//! answers the contract prescribes. The answers come from a lexical scan
//! of the MoonBit expression, and the scan says when it is not exact:
//! a binder form (`fn`, `let`, `match`, `for`, `is`, `=>`, …) or a
//! labelled argument makes [`ExprDialect::bindings_are_exact`] `false`, so
//! the consumer falls back to may-read-anything instead of trusting a
//! guess. Types are `moonc`'s job, never this scan's.

use core::fmt;

use vize_s0::Span;
use vize_s2::expr::ExprRef;
use vize_s2::expr::capability::ExprDialect;

use crate::sfc::DIALECT;
use scan::{Tok, scan};

mod scan;

/// The in-tree MoonBit expression dialect (charter #15's first-party
/// tier: compiled in, statically dispatched, no transport).
#[derive(Debug, Clone, Copy, Default)]
pub struct MoonBitDialect;

/// Keywords that introduce a local binder: names after them may be
/// locals, so an enumeration over such an expression is only a lower
/// bound.
const BINDERS: [&str; 12] = [
    "catch", "fn", "for", "guard", "is", "let", "letrec", "lexmatch", "loop", "match", "try",
    "while",
];

/// The payload text when `expr` is this dialect's.
fn moonbit_source<'a>(expr: ExprRef<'a>) -> Option<&'a str> {
    match expr {
        ExprRef::Foreign(foreign) if foreign.dialect == DIALECT => Some(foreign.source),
        _ => None,
    }
}

/// Free binding names of a MoonBit expression, deduplicated, in source
/// order, and whether that list is exact.
pub(crate) fn free_names(source: &str) -> (Vec<&str>, bool) {
    let Some(tokens) = scan(source) else {
        return (Vec::new(), false);
    };
    let mut names: Vec<&str> = Vec::new();
    let mut exact = true;
    let mut parens = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        let before = index.checked_sub(1).map(|at| tokens[at]);
        let after = tokens.get(index + 1).copied();
        match *token {
            Tok::Keyword(word) if BINDERS.contains(&word) => exact = false,
            Tok::Arrow | Tok::Punct(b'~') => exact = false,
            Tok::Punct(b'(') => parens += 1,
            Tok::Punct(b')') => parens = parens.saturating_sub(1),
            Tok::Name(name) => {
                let member = matches!(before, Some(Tok::Punct(b'.' | b'@') | Tok::PathSep));
                let label = matches!(after, Some(Tok::Punct(b'~')))
                    || (parens > 0 && matches!(after, Some(Tok::Punct(b'='))));
                exact &= !label;
                let local = name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_');
                if !member && !label && local && !names.contains(&name) {
                    names.push(name);
                }
            }
            _ => {}
        }
    }
    (names, exact)
}

/// Whether the expression is a handler *reference* (`save`,
/// `store.save`) rather than an inline handler statement.
#[must_use]
pub fn is_handler_path(source: &str) -> bool {
    let Some(tokens) = scan(source) else {
        return false;
    };
    let mut expect_name = true;
    for token in &tokens {
        match (expect_name, token) {
            (true, Tok::Name(name)) if name.starts_with(|c: char| c.is_ascii_lowercase()) => {}
            (false, Tok::Punct(b'.')) => {}
            _ => return false,
        }
        expect_name = !expect_name;
    }
    !tokens.is_empty() && !expect_name
}

impl ExprDialect for MoonBitDialect {
    fn enumerate_bindings(&self, expr: ExprRef<'_>, each: &mut dyn FnMut(&str)) {
        if let Some(source) = moonbit_source(expr) {
            free_names(source).0.into_iter().for_each(each);
        }
    }

    fn bindings_are_exact(&self, expr: ExprRef<'_>) -> bool {
        moonbit_source(expr).is_some_and(|source| free_names(source).1)
    }

    fn is_constant(&self, expr: ExprRef<'_>) -> bool {
        moonbit_source(expr).and_then(scan).is_some_and(|tokens| {
            tokens.contains(&Tok::Literal)
                && tokens.iter().all(|token| {
                    matches!(
                        token,
                        Tok::Literal | Tok::Punct(b'+' | b'-' | b'*' | b'/' | b'%' | b'(' | b')')
                    )
                })
        })
    }

    fn map_span(&self, expr: ExprRef<'_>, inner: Span) -> Span {
        let span = expr.span();
        let verbatim = u32::try_from(expr.source().len()).is_ok_and(|len| len == span.len());
        if verbatim && inner.start <= inner.end && inner.end <= span.len() {
            Span::new(span.start + inner.start, span.start + inner.end)
        } else {
            span
        }
    }

    fn emit(&self, expr: ExprRef<'_>, out: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        match expr {
            ExprRef::Foreign(foreign) if foreign.dialect == DIALECT => {
                out.write_str(foreign.source)
            }
            // Pessimal law 5: an opaque expression is emitted verbatim or refused.
            ExprRef::Opaque(opaque) => out.write_str(opaque.source),
            ExprRef::Js(_) | ExprRef::Filter(_) | ExprRef::Foreign(_) => Err(fmt::Error),
        }
    }
}
