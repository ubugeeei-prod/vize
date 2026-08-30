//! SFC style-block `v-bind()` → [`BindingOp::VueCssBind`] (P2-10).
//!
//! Style blocks never enter the template [`SurfaceTree`], so this is a
//! parallel admission: CSS in, a carrier [`ElementOp`] whose bindings
//! are the calls. The carrier is not a DOM vnode — P2-11 must skip it.
//! Spans are file-absolute against the complete authored source.

use oxc_span::GetSpan;
use vize_s0::{Allocator, Box, SourceBlock, Span, Vec};
use vize_s2::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};
use vize_s2::op::{BindingOp, ElementOp, Namespace, Op, Region, VueCssBindOp};

mod scan;

/// Lower one style block's `v-bind()` calls onto a carrier `ui.element style`.
///
/// `css` is the style block's **content** (not the wrapping `<style>`
/// tags). `block_start` is that content's file-absolute start.
#[must_use]
pub fn lower_style_block<'a>(allocator: &'a Allocator, css: &'a str, block_start: u32) -> Op<'a> {
    lower_style_block_impl(allocator, css, block_start)
}

/// Lower one validated style source block.
#[must_use]
pub fn lower_style_block_in<'a>(allocator: &'a Allocator, block: SourceBlock<'a>) -> Op<'a> {
    lower_style_block_impl(allocator, block.source(), block.start())
}

fn lower_style_block_impl<'a>(allocator: &'a Allocator, css: &'a str, block_start: u32) -> Op<'a> {
    let mut bindings = Vec::new_in(&allocator);
    let mut pos = 0;
    while let Some(hit) = scan::next(css, pos) {
        let call = authored_span(block_start, hit.call_start, hit.call_end);
        let expr = authored_slice_span(css, hit.expr, block_start);
        bindings.push(BindingOp::VueCssBind(Box::new_in(
            VueCssBindOp {
                value: css_bind_expr(allocator, hit.expr, expr),
                span: call,
            },
            &allocator,
        )));
        pos = hit.call_end;
    }
    let block = authored_span(block_start, 0, css.len());
    Op::Element(Box::new_in(
        ElementOp {
            tag: "style",
            namespace: Namespace::Html,
            attributes: Vec::new_in(&allocator),
            bindings,
            children: Region {
                ops: Vec::new_in(&allocator),
            },
            span: block,
        },
        &allocator,
    ))
}

fn css_bind_expr<'a>(allocator: &'a Allocator, source: &'a str, span: Span) -> ExprRef<'a> {
    match JsExpr::parse_in(allocator, source, span) {
        Ok(js) if js_consumed_without_trailing_comment(js, source) => ExprRef::Js(js),
        Ok(_) => opaque_expr(allocator, source, span, OpaqueReason::ParseRejected),
        Err(reason) => opaque_expr(allocator, source, span, reason),
    }
}

fn js_consumed_without_trailing_comment(js: &JsExpr<'_>, source: &str) -> bool {
    let Some(rest) = source.get(js.ast.span().end as usize..) else {
        return false;
    };
    rest.trim().is_empty()
}

fn opaque_expr<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    span: Span,
    reason: OpaqueReason,
) -> ExprRef<'a> {
    ExprRef::Opaque(allocator.alloc(OpaqueExpr {
        reason,
        source,
        span,
    }))
}

fn authored_span(block_start: u32, start: usize, end: usize) -> Span {
    Span::new(
        block_start.saturating_add(start as u32),
        block_start.saturating_add(end as u32),
    )
}

fn authored_slice_span(css: &str, slice: &str, block_start: u32) -> Span {
    let base = css.as_ptr() as usize;
    let ptr = slice.as_ptr() as usize;
    let start = if ptr >= base && ptr + slice.len() <= base + css.len() {
        ptr - base
    } else {
        0
    };
    authored_span(block_start, start, start + slice.len())
}

impl<'a> super::Lowered<'a> {
    /// Append one style block's carrier to the already-lowered template
    /// tree and keep [`super::Lowered::op_count`] equal to the folio.
    pub fn push_style_block(&mut self, allocator: &'a Allocator, css: &'a str, block_start: u32) {
        let op = lower_style_block(allocator, css, block_start);
        self.push_style_op(op);
    }

    /// Append one validated style block's carrier to the lowered tree.
    pub fn push_style_block_in(&mut self, allocator: &'a Allocator, block: SourceBlock<'a>) {
        let op = lower_style_block_in(allocator, block);
        self.push_style_op(op);
    }

    fn push_style_op(&mut self, op: Op<'a>) {
        let extra = match &op {
            Op::Element(element) if element.bindings.is_empty() => {
                return;
            }
            Op::Element(element) => 1 + element.bindings.len() as u32,
            _ => 1,
        };
        self.root.ops.push(op);
        self.op_count = self.op_count.saturating_add(extra);
    }
}
