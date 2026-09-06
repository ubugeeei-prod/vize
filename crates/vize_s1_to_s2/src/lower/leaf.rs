//! Leaf children: text, interpolations, comments, and the S1 node kinds S2 does
//! not carry (processing instructions, `Unexpected` holes).
//!
//! Dropping is never silent: every non-lowered node leaves a provenance
//! record (the survival law), and the bytes stay recoverable through S1.
//! No *new* diagnostic accompanies a drop — the tokenizer's
//! `SurfaceError`s already cover the malformed spellings (stray end
//! tags, bogus declarations), and S1→S2 must not report the same bytes
//! twice.

use vize_s0::{Box, Span, String, Vec, cstr};
use vize_s1::{Interpolation, SurfaceChild, Token};

use vize_s2::op::{CommentOp, InterpolationOp, Op, TextOp};

use super::cx::Cx;
use super::expr::{desc, filter_expr_at};

/// Lower one non-element child.
pub(crate) fn lower_leaf<'a>(cx: &mut Cx<'a>, child: &SurfaceChild<'a>, out: &mut Vec<'a, Op<'a>>) {
    match child {
        // Elements take the structural path; a slip here is a lowering
        // bug, and totality still holds (the child is skipped, never a
        // panic outside debug builds).
        SurfaceChild::Element(_) => debug_assert!(false, "elements take the structural path"),
        SurfaceChild::Text(token) => lower_text(cx, token, token.text, out),
        SurfaceChild::Interpolation(node) => lower_interpolation(cx, node, out),
        SurfaceChild::Comment(token) => {
            if cx.preserve_comments() {
                lower_comment(cx, token, out);
            } else {
                let span = cx.token_span(token);
                cx.record("drop.comment", None, token.text, String::default(), span);
            }
        }
        SurfaceChild::Cdata(token) => lower_cdata(cx, token, out),
        SurfaceChild::ProcessingInstruction(token) => {
            let span = cx.token_span(token);
            cx.record(
                "drop.processing-instruction",
                None,
                token.text,
                String::default(),
                span,
            );
        }
        SurfaceChild::Unexpected(token) => {
            let span = cx.token_span(token);
            cx.record("drop.unexpected", None, token.text, String::default(), span);
        }
    }
}

/// `ui.text`. `content` is the rendered text — the authored bytes, or
/// the condensed rewrite the P2-9 text plan decided (`lower::text`);
/// entity decoding stays out of scope (the S1 v1 no-decoding deviation,
/// re-recorded in the installment-4 record).
pub(crate) fn lower_text<'a>(
    cx: &mut Cx<'a>,
    token: &Token<'a>,
    content: &'a str,
    out: &mut Vec<'a, Op<'a>>,
) {
    let node = cx.mint_op();
    let span = cx.token_span(token);
    cx.record(
        "lower.text",
        node,
        token.text,
        String::from("ui.text"),
        span,
    );
    out.push(Op::Text(Box::new_in(
        TextOp { content, span },
        &cx.allocator,
    )));
}

/// `ui.interpolation`: the delimited content, trimmed, through the total
/// admission rule.
fn lower_interpolation<'a>(cx: &mut Cx<'a>, node: &Interpolation<'a>, out: &mut Vec<'a, Op<'a>>) {
    let id = cx.mint_op();
    let span = Span::new(cx.offset(node.open.text), cx.token_span(&node.close).end);
    let expression = filter_expr_at(cx, node.content.text);
    cx.record(
        "lower.interpolation",
        id,
        node.content.text,
        cstr!("ui.interpolation {}", desc(&expression)),
        span,
    );
    out.push(Op::Interpolation(Box::new_in(
        InterpolationOp { expression, span },
        &cx.allocator,
    )));
}

fn lower_comment<'a>(cx: &mut Cx<'a>, token: &Token<'a>, out: &mut Vec<'a, Op<'a>>) {
    let node = cx.mint_op();
    let span = cx.token_span(token);
    cx.record(
        "lower.comment",
        node,
        token.text,
        String::from("ui.comment"),
        span,
    );
    out.push(Op::Comment(Box::new_in(
        CommentOp {
            content: comment_content(token.text),
            span,
        },
        &cx.allocator,
    )));
}

fn comment_content(text: &str) -> &str {
    text.strip_prefix("<!--")
        .and_then(|inner| inner.strip_suffix("-->"))
        .unwrap_or(text)
}

/// A CDATA section's content is text (the foreign-namespace reading);
/// the framing is dropped with the record naming the rule.
fn lower_cdata<'a>(cx: &mut Cx<'a>, token: &Token<'a>, out: &mut Vec<'a, Op<'a>>) {
    let inner = token.text.strip_prefix("<![CDATA[").unwrap_or(token.text);
    let inner = inner.strip_suffix("]]>").unwrap_or(inner);
    let node = cx.mint_op();
    let span = cx.span_of(inner);
    cx.record(
        "lower.cdata-text",
        node,
        token.text,
        String::from("ui.text"),
        span,
    );
    out.push(Op::Text(Box::new_in(
        TextOp {
            content: inner,
            span,
        },
        &cx.allocator,
    )));
}
