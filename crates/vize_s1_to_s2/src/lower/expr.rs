//! Expression admission for the lowering: every expression position goes
//! through P2-5b's total [`ExprRef`] contract.
//!
//! The lowering **never re-derives the admission rule**: text goes
//! through [`ExprRef::parse_js_in`] — the one shared home of the P1-5
//! guard → parse → whole-coverage rule — and comes back `Js` when
//! admitted or `Opaque` with the text-classified reason when not. Vue 2
//! pipe filters are an exception that is still total: interpolations
//! and `v-bind` values run [`VueFilterExpr::parse_in`] first so `|` is
//! not bitwise-OR. Other positions (`v-on`, dynamic arguments, …) stay
//! on the JS rule — Vue 2 never treated those as filter sites.
//! The position-classified reasons (`ForValue` here; `MultiStatement` and
//! `Compound` have no P2-8 producer, see the record) are assigned by
//! [`opaque_at`], the only constructor that names a reason directly.

use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::{Span, String, cstr};
use vize_s2::expr::{ExprRef, OpaqueExpr, OpaqueReason, VueFilterExpr};

use super::cx::Cx;

/// Trim `text` and return the trimmed slice with its authored span.
/// Trimming keeps the slice a source slice, so spans stay exact.
pub(crate) fn trimmed<'a>(cx: &Cx<'a>, text: &'a str) -> (&'a str, Span) {
    let slice = text.trim();
    (slice, cx.span_of(slice))
}

/// Lower one expression position: trim, then admit through the shared
/// JS rule. Total — refused text comes back as the classified escape.
pub(crate) fn expr_at<'a>(cx: &Cx<'a>, text: &'a str) -> ExprRef<'a> {
    let (slice, span) = trimmed(cx, text);
    ExprRef::parse_js_in(cx.allocator, slice, span)
}

/// `v-on` handler admission: a lone expression keeps the retained JS
/// payload, while a valid statement body that cannot be represented by one
/// expression is classified explicitly as `Opaque(MultiStatement)`.
pub(crate) fn handler_expr_at<'a>(cx: &Cx<'a>, text: &'a str) -> ExprRef<'a> {
    let expr = expr_at(cx, text);
    let ExprRef::Opaque(opaque) = expr else {
        return expr;
    };
    if opaque.reason != OpaqueReason::ParseRejected || !parses_as_program(cx, opaque.source) {
        return expr;
    }
    opaque_at(cx, OpaqueReason::MultiStatement, opaque.source, opaque.span)
}

fn parses_as_program(cx: &Cx<'_>, source: &str) -> bool {
    Parser::new(
        cx.allocator.as_oxc(),
        source,
        SourceType::ts().with_module(true),
    )
    .parse()
    .diagnostics
    .is_empty()
}

/// Interpolation / `v-bind` value admission: Vue 2 pipe filters first
/// when the dialect asks, so `|` is not bitwise-OR. Other directive
/// expressions stay on [`expr_at`].
pub(crate) fn filter_expr_at<'a>(cx: &Cx<'a>, text: &'a str) -> ExprRef<'a> {
    let (slice, span) = trimmed(cx, text);
    if cx.caps.supports_filters
        && let Some(filter) = VueFilterExpr::parse_in(cx.allocator, slice, span)
    {
        return ExprRef::Filter(filter);
    }
    ExprRef::parse_js_in(cx.allocator, slice, span)
}

/// A position-classified escape at an exact place: the reasons only the
/// lowering can assign, because they are facts about where the text came
/// from, never recoverable from the text (P2-5b).
pub(crate) fn opaque_at<'a>(
    cx: &Cx<'a>,
    reason: OpaqueReason,
    slice: &'a str,
    span: Span,
) -> ExprRef<'a> {
    ExprRef::Opaque(cx.allocator.alloc(OpaqueExpr {
        reason,
        source: slice,
        span,
    }))
}

/// The provenance spelling of an admitted reference: `js`, `foreign`, or
/// `opaque(<reason>)` — the mnemonic vocabulary the folio uses.
pub(crate) fn desc(expr: &ExprRef<'_>) -> String {
    match expr {
        ExprRef::Js(_) | ExprRef::Foreign(_) => String::from(expr.mnemonic()),
        ExprRef::Filter(_) => String::from("vue.filter"),
        ExprRef::Opaque(opaque) => cstr!("opaque({})", opaque.reason.mnemonic()),
    }
}

/// The identifier an admitted expression binds, when it is exactly one:
/// the only name enumeration P2-8 does. Patterns return `None` — the
/// single identifier-extraction implementation is the
/// `ExprDialect::enumerate_bindings` seam (#4365), not a scanner here.
pub(crate) fn simple_identifier<'a>(expr: &ExprRef<'a>) -> Option<&'a str> {
    match expr {
        ExprRef::Js(js) => match js.ast {
            oxc_ast::ast::Expression::Identifier(_) => Some(js.source),
            _ => None,
        },
        ExprRef::Foreign(_) | ExprRef::Filter(_) | ExprRef::Opaque(_) => None,
    }
}
