//! Language-owned expression payload printing for the stable L2 folio.

use core::fmt::{Result, Write};

use crate::dump::Expr;
use crate::dump::print::{Style, quoted, span_tail};

/// Write one expression payload in the selected folio style.
pub(crate) fn print_expr<W: Write>(w: &mut W, expr: &Expr, mode: Style) -> Result {
    let (head, source, span) = match expr {
        Expr::Js { source, span } => {
            w.write_str("js(")?;
            ("", source, span)
        }
        Expr::Opaque {
            reason,
            source,
            span,
        } => {
            w.write_str("opaque(")?;
            (reason.mnemonic(), source, span)
        }
        Expr::Foreign {
            dialect,
            source,
            span,
        } => {
            w.write_str("foreign(")?;
            (dialect.as_str(), source, span)
        }
        Expr::Filter { source, span } => {
            w.write_str("vue.filter(")?;
            ("", source, span)
        }
    };
    if !head.is_empty() {
        w.write_str(head)?;
        w.write_char(' ')?;
    }
    quoted(w, source.as_str())?;
    span_tail(w, *span, mode)?;
    w.write_char(')')
}
