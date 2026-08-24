//! SFC style-block `v-bind()` → [`BindingOp::VueCssBind`] (P2-10).
//!
//! Style blocks never enter the template [`SurfaceTree`], so this is a
//! parallel admission: CSS in, a carrier [`ElementOp`] whose bindings
//! are the calls. The carrier is not a DOM vnode — P2-11 must skip it.
//! Spans are **block-relative** via [`Span::to_block_relative`].

use vize_s0::{Allocator, Box, Span, Vec};
use vize_s2::expr::ExprRef;
use vize_s2::op::{BindingOp, ElementOp, Namespace, Op, Region, VueCssBindOp};

mod scan;

/// Lower one style block's `v-bind()` calls onto a carrier `ui.element style`.
///
/// `css` is the style block's **content** (not the wrapping `<style>`
/// tags). `block_start` is that content's file-absolute start; every
/// span on the result is relative to it.
#[must_use]
pub fn lower_style_block<'a>(allocator: &'a Allocator, css: &'a str, block_start: u32) -> Op<'a> {
    let mut bindings = Vec::new_in(&allocator);
    let mut pos = 0;
    while let Some(hit) = scan::next(css, pos) {
        let call = rebase(block_start, hit.call_start, hit.call_end);
        let expr = rebase_slice(css, hit.expr, block_start);
        bindings.push(BindingOp::VueCssBind(Box::new_in(
            VueCssBindOp {
                value: ExprRef::parse_js_in(allocator, hit.expr, expr),
                span: call,
            },
            &allocator,
        )));
        pos = hit.call_end;
    }
    let block = rebase(block_start, 0, css.len());
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

fn rebase(block_start: u32, start: usize, end: usize) -> Span {
    Span::new(
        block_start.saturating_add(start as u32),
        block_start.saturating_add(end as u32),
    )
    .to_block_relative(block_start)
}

/// File-absolute span of `slice` inside `css`, then block-relative.
fn rebase_slice(css: &str, slice: &str, block_start: u32) -> Span {
    let base = css.as_ptr() as usize;
    let ptr = slice.as_ptr() as usize;
    let start = if ptr >= base && ptr + slice.len() <= base + css.len() {
        ptr - base
    } else {
        0
    };
    rebase(block_start, start, start + slice.len())
}

impl<'a> super::Lowered<'a> {
    /// Append one style block's carrier to the already-lowered template
    /// tree and keep [`super::Lowered::op_count`] equal to the folio.
    pub fn push_style_block(&mut self, allocator: &'a Allocator, css: &'a str, block_start: u32) {
        let op = lower_style_block(allocator, css, block_start);
        let extra = match &op {
            Op::Element(element) => 1 + element.bindings.len() as u32,
            _ => 1,
        };
        self.root.ops.push(op);
        self.op_count = self.op_count.saturating_add(extra);
    }
}
