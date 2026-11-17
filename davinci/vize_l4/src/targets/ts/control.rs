//! `ui.if` and `ui.for` authored as script: `&&`, `?:` and `.map()`.
//!
//! The op keeps its authored bytes: each condition is one expression link,
//! the rest are script runs (the operators, an authored callback head and its
//! body around the returned markup). Each region is projected where its op
//! was authored. An element region becomes the array of its items, and a
//! nested `ui.if` / `ui.for` is projected the same way in place.

use vize_l0::Span;
use vize_l2::op::{Op, Region};

use super::module::{Cx, JsxModuleRefusal};
use super::ops::{self, Items};
use super::output::LinkRole;

/// Project a `ui.if` / `ui.for` in child position, whose span is its
/// expression container `{…}`.
pub(super) fn child(cx: &mut Cx<'_>, op: &Op<'_>) -> Result<(), JsxModuleRefusal> {
    let span = span_of(op)?;
    let brace = |at: u32| cx.source.get(at as usize..(at as usize).saturating_add(1));
    if span.end.saturating_sub(span.start) < 2
        || brace(span.start) != Some("{")
        || brace(span.end - 1) != Some("}")
    {
        return Err(JsxModuleRefusal::Span);
    }
    in_place(cx, op, span, Span::new(span.start + 1, span.end - 1))
}

/// Write the authored expression `inner` of `op` (owned by `authored`).
fn in_place(
    cx: &mut Cx<'_>,
    op: &Op<'_>,
    authored: Span,
    inner: Span,
) -> Result<(), JsxModuleRefusal> {
    cx.anchor(authored, LinkRole::Expression, |cx| {
        let mut cursor = inner.start;
        match op {
            Op::If(conditional) => {
                for branch in &conditional.branches {
                    if let Some(condition) = &branch.condition {
                        let at = condition.span();
                        gap(cx, cursor, at.start, inner)?;
                        ops::expression(cx, condition)?;
                        cursor = at.end;
                    }
                    cursor = region(cx, &branch.region, cursor, inner)?;
                }
            }
            // An authored callback head: no host loop.
            Op::For(each) if each.host.is_none() => {
                cursor = region(cx, &each.region, cursor, inner)?;
            }
            _ => return Err(JsxModuleRefusal::Unsupported(op.mnemonic())),
        }
        gap(cx, cursor, inner.end, inner)
    })
}

/// Project the one op a region of authored script holds, after the script
/// before it. Returns where the op ends.
fn region(
    cx: &mut Cx<'_>,
    region: &Region<'_>,
    cursor: u32,
    inner: Span,
) -> Result<u32, JsxModuleRefusal> {
    let [op] = region.ops.as_slice() else {
        return Err(JsxModuleRefusal::Span);
    };
    let span = span_of(op)?;
    gap(cx, cursor, span.start, inner)?;
    match op {
        Op::Element(element) => cx.anchor(span, LinkRole::Expression, |cx| {
            cx.out.push("[");
            ops::element(cx, element, &mut Items::new())?;
            cx.out.push("]");
            Ok(())
        })?,
        Op::If(_) | Op::For(_) => in_place(cx, op, span, span)?,
        _ => return Err(JsxModuleRefusal::Unsupported(op.mnemonic())),
    }
    Ok(span.end)
}

/// The authored script between `from` and `to`, both inside `inner`.
fn gap(cx: &mut Cx<'_>, from: u32, to: u32, inner: Span) -> Result<(), JsxModuleRefusal> {
    if from > to || to > inner.end {
        return Err(JsxModuleRefusal::Span);
    }
    cx.script(Span::new(from, to))
}

fn span_of(op: &Op<'_>) -> Result<Span, JsxModuleRefusal> {
    match op {
        Op::Element(element) => Ok(element.span),
        Op::If(conditional) => Ok(conditional.span),
        Op::For(each) => Ok(each.span),
        // The owner-object helper for components is not published yet.
        Op::Component(_) => Err(JsxModuleRefusal::UnpublishedHelper(op.mnemonic())),
        _ => Err(JsxModuleRefusal::Unsupported(op.mnemonic())),
    }
}
