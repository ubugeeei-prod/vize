//! Keep original decoding geometry beside the compatibility lane's one AST.

use super::{Cx, ExprRef, OpaqueReason, VueFilterExpr, expression_in, opaque_at};
use crate::expression_coordinates::expression_coordinates;
use vize_l0::Span;
use vize_l1::embed::prepare_attribute_value;
use vize_l1::markup::entity::needs_decoding;
use vize_l2::expr::JsExpr;

#[cfg(test)]
mod tests;

pub(super) fn expression<'a>(cx: &Cx<'a>, raw: &'a str, span: Span, filters: bool) -> ExprRef<'a> {
    // Retain the current reference-free path and allocation count.
    if !needs_decoding(raw.as_bytes()) {
        return plain(cx, raw, span, filters);
    }
    if cx
        .source
        .get(span.start as usize..span.end as usize)
        .is_none_or(|original| !core::ptr::eq(original, raw))
    {
        return opaque_at(cx, OpaqueReason::SourceCoordinates, raw, span);
    }
    let Ok(source) = prepare_attribute_value(cx.allocator, cx.source, span) else {
        return opaque_at(cx, OpaqueReason::SourceCoordinates, raw, span);
    };
    let text = source.text();
    if cx.foreign_dialect.is_some() || source.decode_map().is_none() {
        return plain(cx, text, span, filters);
    }
    if let Some(filter) = filter(cx, text, span, filters) {
        return filter;
    }
    let Ok(coordinates) = expression_coordinates(cx.allocator, cx.source, source, 0) else {
        return opaque_at(cx, OpaqueReason::SourceCoordinates, text, span);
    };
    // Same complete-expression parser/guard as ExprRef::parse_js_in, exactly
    // once. Allocate one JsExpr after pairing its existing AST with the map.
    let ast = match JsExpr::parse_ast_in(cx.allocator, text) {
        Ok(ast) => cx.allocator.alloc(ast),
        Err(reason) => return opaque_at(cx, reason, text, span),
    };
    match JsExpr::from_retained_in(cx.allocator, ast, text, span, coordinates) {
        Ok(js) => ExprRef::Js(js),
        Err(_) => opaque_at(cx, OpaqueReason::SourceCoordinates, text, span),
    }
}

fn plain<'a>(cx: &Cx<'a>, text: &'a str, span: Span, filters: bool) -> ExprRef<'a> {
    filter(cx, text, span, filters).unwrap_or_else(|| expression_in(cx, text, span))
}

fn filter<'a>(cx: &Cx<'a>, text: &'a str, span: Span, filters: bool) -> Option<ExprRef<'a>> {
    if filters && let Some(filter) = VueFilterExpr::parse_in(cx.allocator, text, span) {
        return Some(ExprRef::Filter(filter));
    }
    None
}
