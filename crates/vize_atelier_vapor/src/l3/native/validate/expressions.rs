//! Retained JavaScript operands consumed without expression reparsing.

use super::super::Expr;
use super::{
    Result,
    ident::{path_root, reference, trimmed},
};
use crate::l3::{LegacyReason, retained::Retained};
use vize_l3::operand::{Operand, ValueKind};

/// A JavaScript operand the generator can resolve without reparsing: a direct
/// reference, or an expression whose retained AST moved into the output arena.
pub(super) fn js<'a>(retained: &Retained<'_, 'a>, operand: &Operand<'a>) -> Result<Expr<'a>> {
    expression(retained, operand, false)
}

/// [`js`] for an event handler. A direct reference takes the generator's
/// simple-path fast path elsewhere, but a component handler is classified
/// first; its retained AST, when L2 has one, keeps that parse-free.
pub(super) fn handler<'a>(retained: &Retained<'_, 'a>, operand: &Operand<'a>) -> Result<Expr<'a>> {
    expression(retained, operand, true)
}

fn expression<'a>(
    retained: &Retained<'_, 'a>,
    operand: &Operand<'a>,
    classified: bool,
) -> Result<Expr<'a>> {
    let value = operand.value;
    if value.kind != ValueKind::Js || context_reserved(value.text) {
        return Err(LegacyReason::ExpressionOrEncoding.into());
    }
    if reference(value.text) {
        return Ok(
            match classified
                .then(|| retained.expression(value.text, value.span))
                .flatten()
            {
                Some(js) => Expr {
                    text: value.text,
                    js: Some(js),
                },
                None => Expr::plain(trimmed(value.text)),
            },
        );
    }
    // `$event`-rooted paths stay on the legacy lane (see the P3-6 record).
    if path_root(value.text) == Some("$event") {
        return Err(LegacyReason::ExpressionOrEncoding.into());
    }
    let js = retained
        .expression(value.text, value.span)
        .ok_or(LegacyReason::ExpressionOrEncoding)?;
    Ok(Expr {
        text: value.text,
        js: Some(js),
    })
}

/// The shared generator leaves these roots bare while the retained lane's
/// prefixing rewrites them onto `_ctx`; the lanes would observe different
/// bindings. The textual test over-approximates (it also matches strings).
fn context_reserved(text: &str) -> bool {
    // Every reserved root starts with `_` or `$`; most expressions have none.
    text.bytes().any(|b| b == b'_' || b == b'$')
        && ["_ctx", "$props", "$attrs", "$slots", "$emit"]
            .iter()
            .any(|name| text.contains(name))
}
