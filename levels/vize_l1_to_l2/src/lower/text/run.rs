use vize_l0::{Box, StringBuilder, cstr};
use vize_l1::markup::SurfaceChild;
use vize_l2::expr::OpaqueReason;
use vize_l2::op::{InterpolationOp, Op, TextOp};
use vize_l2::provenance::ProvenanceSink;

use super::lane::{RunShape, TextRun};
use super::{TextAction, TextParts, rebuild_source};
use crate::lower::cx::Cx;
use crate::lower::expr::{desc, filter_expr_at, opaque_at};

/// Record what a run consumed without lowering it, in scan order.
fn record_consumed(cx: &mut Cx<'_, impl ProvenanceSink>, run: &TextRun<'_>) {
    for consumed in &run.consumed {
        cx.record(
            consumed.kind,
            None,
            consumed.text,
            vize_l0::String::default,
            consumed.span,
        );
    }
}

/// Lower the maximal text/interpolation run starting at `start`,
/// returning the index just past it. Dropped whitespace lowers to
/// nothing (recorded); a one-node run lowers as the plain leaf; a
/// longer run merges; a `v-pre` run is one text.
pub(crate) fn lower_text_run<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    children: &[SurfaceChild<'a>],
    plan: &[TextAction<'a>],
    start: usize,
    out: &mut vize_l0::Vec<'a, Op<'a>>,
) -> usize {
    let run = cx.text_lane().run(children, plan, start);
    let end = run.end;
    record_consumed(cx, &run);
    match run.shape {
        RunShape::Dropped => {}
        RunShape::Text { content, .. } => {
            if let Some(SurfaceChild::Text(token)) = children.get(start) {
                if content != token.text {
                    let span = cx.token_span(token);
                    cx.record(
                        "condense.whitespace",
                        None,
                        token.text,
                        || vize_l0::String::from(content),
                        span,
                    );
                }
                crate::lower::leaf::lower_text(cx, token, content, out);
            }
        }
        RunShape::Interpolation { .. } => {
            if let Some(child) = children.get(start) {
                crate::lower::leaf::lower_leaf(cx, child, out);
            }
        }
        RunShape::Frozen { content, span } => {
            let raw = cx
                .source
                .get(span.start as usize..span.end as usize)
                .unwrap_or_default();
            let mut interned = StringBuilder::with_capacity_in(content.len(), cx.allocator);
            interned.push_str(content.as_str());
            let node = cx.mint_op();
            cx.record(
                "lower.v-pre-text",
                node,
                raw,
                || vize_l0::String::from("ui.text"),
                span,
            );
            out.push(Op::Text(Box::new_in(
                TextOp {
                    content: interned.into_str(),
                    span,
                },
                &cx.allocator,
            )));
        }
        RunShape::Merged {
            parts,
            span,
            members,
        } => {
            let raw = cx
                .source
                .get(span.start as usize..span.end as usize)
                .unwrap_or_default();
            let dynamic = parts.iter().filter(|part| part.dynamic).count();
            let node = cx.mint_op();
            if dynamic == 0 {
                // A text-only run merges into one `ui.text`.
                let mut content = StringBuilder::with_capacity_in(raw.len(), cx.allocator);
                for part in &parts {
                    content.push_str(part.text.as_str());
                }
                cx.record(
                    "lower.text-run",
                    node,
                    raw,
                    || cstr!("ui.text merged={members}"),
                    span,
                );
                out.push(Op::Text(Box::new_in(
                    TextOp {
                        content: content.into_str(),
                        span,
                    },
                    &cx.allocator,
                )));
            } else {
                // The compiler lanes never consume the compatibility ASTs.
                // Retain them only for the comment-preserving lint lowering;
                // otherwise every mixed run would parse each expression a
                // second time after the compound representation is built.
                if cx.preserve_comments() {
                    for child in children.get(start..end).unwrap_or_default() {
                        if let SurfaceChild::Interpolation(interpolation) = child {
                            let expression = filter_expr_at(cx, interpolation.content.text);
                            cx.consumed_expressions.push(expression);
                        }
                    }
                }
                // A mixed run is the compound representation: one
                // `ui.interpolation` under the pessimal opaque laws, parts
                // recorded beside the tree for the pass to validate and P2-11
                // to compile from.
                let rebuilt = rebuild_source(&parts);
                let mut source = StringBuilder::with_capacity_in(rebuilt.len(), cx.allocator);
                source.push_str(rebuilt.as_str());
                let expression = opaque_at(cx, OpaqueReason::Compound, source.into_str(), span);
                let count = parts.len();
                cx.record(
                    "lower.compound",
                    node,
                    raw,
                    || {
                        cstr!(
                            "ui.interpolation {} parts={} dynamic={}",
                            desc(&expression),
                            count,
                            dynamic
                        )
                    },
                    span,
                );
                cx.attach_texts(node, TextParts { parts }, span, rebuilt.as_str());
                out.push(Op::Interpolation(Box::new_in(
                    InterpolationOp {
                        expression,
                        raw_html: false,
                        span,
                    },
                    &cx.allocator,
                )));
            }
        }
    }
    end
}
