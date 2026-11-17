//! Python on the Lang axis, parsed with RustPython and lowered to JavaScript.
//!
//! RustPython recognizes the complete Python grammar and supplies source
//! ranges. This example deliberately lowers a smaller, documented semantic
//! surface and refuses every other AST node at its precise range.

mod emit;
mod script;

use poc_support::Error;
use rustpython_parser::ast;
use vize_l0::Span;
#[cfg(test)]
use vize_l0::String;
use vize_l2::expr::{ExprDialect, ExprRef};

pub(crate) use script::script;

/// The `python` expression capability the levels bind once per file.
#[derive(Debug, Default)]
pub(crate) struct Python;

impl ExprDialect for Python {
    fn enumerate_bindings(&self, expr: ExprRef<'_>, each: &mut dyn FnMut(&str)) {
        if let Ok(parsed) = emit::parse(expr.source()) {
            emit::enumerate_bindings(&parsed, each);
        }
    }

    fn bindings_are_exact(&self, expr: ExprRef<'_>) -> bool {
        emit::parse(expr.source()).is_ok_and(|parsed| emit::is_supported(&parsed))
    }

    fn is_constant(&self, expr: ExprRef<'_>) -> bool {
        emit::parse(expr.source()).is_ok_and(|parsed| emit::is_constant(&parsed))
    }

    fn map_span(&self, expr: ExprRef<'_>, inner: Span) -> Span {
        let start = expr.span().start;
        Span::new(start + inner.start, start + inner.end)
    }

    fn emit(&self, expr: ExprRef<'_>, out: &mut dyn core::fmt::Write) -> core::fmt::Result {
        let parsed = emit::parse(expr.source()).map_err(|_| core::fmt::Error)?;
        emit::write_expression(&parsed, out).map_err(|_| core::fmt::Error)
    }
}

#[cfg(test)]
fn expression(source: &str) -> Result<String, ()> {
    let parsed = emit::parse(source)?;
    emit::expression(&parsed)
}

fn unsupported(_source: &str, base: u32, range: ast::text_size::TextRange) -> Error {
    let start = u32::from(range.start());
    let end = u32::from(range.end()).max(start.saturating_add(1));
    Error::new(
        "unsupported Python syntax",
        Span::new(base + start, base + end),
    )
}

#[cfg(test)]
mod tests;
