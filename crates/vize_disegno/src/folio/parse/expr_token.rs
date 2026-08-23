//! Expression-payload token grammar (P2-5b; Vue 2 filters P2-9):
//! `js(<quoted> @s:e)`, `opaque(<reason> <quoted> @s:e)`,
//! `foreign(<dialect> <quoted> @s:e)`, `vue.filter(<quoted> @s:e)`.
//!
//! The payload is owned text + span, never an AST - a `js(...)` token
//! re-parses into the arena on load (see `crate::expr::js`). Reasons are
//! the closed [`OpaqueReason`] spellings; dialect ids are bare non-empty
//! words (quotes, spaces and `)` in a dialect id are outside the
//! contract, the same documented-edges rule the quoted-string grammar
//! uses).

use vize_carton::{Span, String, cstr};
use vize_davinci::folio::FolioError;

use super::super::owned::FolioExpr;
use super::line::{err, final_span, take_quoted};
use crate::expr::OpaqueReason;

/// Parse one expression payload starting at `rest[0]`; returns the owned
/// payload and the remainder after the closing `)`.
pub(super) fn take_expr(rest: &str, line_no: usize) -> Result<(FolioExpr, &str), FolioError> {
    if let Some(tail) = rest.strip_prefix("js(") {
        let (source, tail) = take_quoted(tail, line_no)?;
        let (span, tail) = close_span(tail, line_no)?;
        return Ok((FolioExpr::Js { source, span }, tail));
    }
    if let Some(tail) = rest.strip_prefix("opaque(") {
        let Some((word, tail)) = tail.split_once(' ') else {
            return Err(err(line_no, cstr!("expected an opaque reason")));
        };
        let Some(reason) = OpaqueReason::from_mnemonic(word) else {
            return Err(err(line_no, cstr!("unknown opaque reason `{word}`")));
        };
        let (source, tail) = take_quoted(tail, line_no)?;
        let (span, tail) = close_span(tail, line_no)?;
        return Ok((
            FolioExpr::Opaque {
                reason,
                source,
                span,
            },
            tail,
        ));
    }
    if let Some(tail) = rest.strip_prefix("foreign(") {
        let Some((dialect, tail)) = tail.split_once(' ') else {
            return Err(err(line_no, cstr!("expected a dialect id")));
        };
        if dialect.is_empty() {
            return Err(err(line_no, cstr!("expected a dialect id")));
        }
        let (source, tail) = take_quoted(tail, line_no)?;
        let (span, tail) = close_span(tail, line_no)?;
        return Ok((
            FolioExpr::Foreign {
                dialect: String::from(dialect),
                source,
                span,
            },
            tail,
        ));
    }
    if let Some(tail) = rest.strip_prefix("vue.filter(") {
        let (source, tail) = take_quoted(tail, line_no)?;
        let (span, tail) = close_span(tail, line_no)?;
        return Ok((FolioExpr::Filter { source, span }, tail));
    }
    Err(err(line_no, cstr!("expected an expression payload")))
}

/// Parse the ` @s:e)` that closes an expression payload; returns the span
/// and the remainder after `)`.
fn close_span(tail: &str, line_no: usize) -> Result<(Span, &str), FolioError> {
    let Some(tail) = tail.strip_prefix(' ') else {
        return Err(err(line_no, cstr!("missing span")));
    };
    let Some(close) = tail.find(')') else {
        return Err(err(line_no, cstr!("unterminated expression payload")));
    };
    let (body, after) = tail.split_at(close);
    let span = final_span(body, line_no)?;
    Ok((span, &after[1..]))
}
