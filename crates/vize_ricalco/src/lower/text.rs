//! Whitespace condensing and text/interpolation merging — the P2-9
//! installment-4 port of the legacy text lane, absorbed into the
//! lowering.
//!
//! # Why this lives in the lowering, not a pass (the recorded decision)
//!
//! The legacy lane runs both halves *outside* its transform directory:
//! whitespace condensing at **parse time**
//! (`crates/vize_armature/src/parser/whitespace.rs`, driven by the
//! shipped `is_pre_tag`) and text/interpolation merging at **codegen
//! time** (`crates/vize_atelier_core/src/codegen/children.rs`, the
//! consecutive-run grouping); the old step file
//! (`transforms/transform_text.rs`) is exported but never called by the
//! shipped pipeline. The S2 port pulls both into the S1→S2 conversion,
//! for one decisive reason: **comments**. Both computations read comment
//! positions — a comment is a non-text-like neighbour for the
//! remove-vs-condense rule and a hard boundary for run merging — and
//! comments exist only in S1 (the lowering drops them under
//! `drop.comment`). A post-lowering pass would be comment-blind and
//! wrong on real shapes (`a<!--c-->\n<!--d-->b` must lose its
//! whitespace, `a<!--c-->b` must stay two text units); the lowering is
//! the last stage that can compute the legacy answers. P2-5b's record
//! independently requires it: the position-classified opaque reasons —
//! [`OpaqueReason::Compound`] included — "are assignable only by the
//! S1→S2 lowering".
//!
//! # Condensing (Vue's `condense` strategy, the armature algorithm)
//!
//! Rule-for-rule the shipped `condense_whitespace`
//! (`whitespace.rs:69-165`), over the S1 child list of every lowered
//! region: leading/trailing whitespace-only text removed; a
//! whitespace-only run between two non-text-like neighbours with a
//! newline removed, condensed to one space otherwise; interior runs of
//! the Vue alphabet `[ \t\n\f\r]` in mixed text collapsed to one space.
//! Exemptions follow the shipped DOM configuration and the S2 artifact's
//! own soundness: `<pre>` subtrees (`is_pre_tag`,
//! `crates/vize_atelier_dom/src/compile/stage_options.rs`) and rawtext
//! content elements ([`RAWTEXT_TAGS`] — a `<script>`'s bytes are another
//! language, and collapsing them would change JS semantics via ASI).
//!
//! # Merging, and the first `Compound` producer
//!
//! A maximal run of list-adjacent, span-contiguous text/interpolation
//! children lowers to **one op**: a text-only run to one `ui.text`
//! (concatenated content — natural census zero, the tokenizer emits
//! maximal runs; the lane exists for comment-punched and split trees),
//! and a mixed run to one `ui.interpolation` whose expression is
//! [`OpaqueReason::Compound`] — the escape class P2-5b reserved, gaining
//! its first producer here. The **pessimal laws apply from the first
//! byte** (`vize_s2::expr::opaque`): a compound is never constant,
//! equal to nothing, and emittable only byte-verbatim-or-refusal — so
//! the rebuilt [`OpaqueExpr::source`] is a *display* form, and DOM
//! realization (P2-11) compiles from the recorded parts instead
//! ([`TextParts`], validated and published by `pass::text`). The rebuild
//! spelling is canonical template syntax — static parts verbatim,
//! dynamic parts as `{{ <trimmed> }}` — shared verbatim with the pass's
//! re-derivation ([`rebuild_source`], the one-rule-two-sides
//! discipline). Merging never crosses a comment, a dropped node, or any
//! source gap: runs extend only over span-contiguous children, so the
//! merged span is the authored bytes exactly.
//!
//! [`OpaqueExpr::source`]: vize_s2::expr::OpaqueExpr::source

use alloc::vec::Vec as StdVec;

use vize_s0::{Box, Span, String, StringBuilder, cstr};
use vize_s1::SurfaceChild;
use vize_s2::expr::OpaqueReason;
use vize_s2::op::{InterpolationOp, Op, TextOp};

use super::cx::Cx;
use super::expr::{desc, opaque_at, trimmed};

mod condense;

use condense::collapse_fused;
use condense::extends_run;
pub(crate) use condense::{TextAction, plan_whitespace, suppresses_condense};

/// One part of a merged run, owned (the fact crosses compile boundaries
/// with its artifact, P1-11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPart {
    /// The part's rendered text: condensed content for a static part,
    /// the trimmed expression for a dynamic one.
    pub text: String,
    /// The authored range (the raw token for a static part, the whole
    /// `{{ … }}` for a dynamic one).
    pub span: Span,
    /// Whether the part renders an expression.
    pub dynamic: bool,
}

/// The recorded parts of one merged mixed run, keyed by the compound
/// `ui.interpolation` op's id ([`crate::lower::Lowered::texts`]). The
/// `pass::text` consumption validates every entry against the op it
/// keys and publishes the consumed view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextParts {
    /// The parts, in document order.
    pub parts: StdVec<TextPart>,
}

const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<TextPart>();
    assert_owned::<TextParts>();
};

/// 64-bit footprints, guarded like every node-size assert (the wasm32
/// lane is 32-bit).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<TextPart>() == 40);
    assert!(core::mem::size_of::<TextParts>() == 24);
};

/// The rebuild rule: the canonical template spelling of a merged run —
/// static parts verbatim, dynamic parts as `{{ <text> }}`. One home,
/// used by the lowering to mint [`OpaqueExpr::source`] and by the pass
/// to re-derive it (`pass::text`), so the two can only disagree when one
/// of them changed.
///
/// [`OpaqueExpr::source`]: vize_s2::expr::OpaqueExpr::source
pub fn rebuild_source(parts: &[TextPart]) -> String {
    let mut out = String::default();
    for part in parts {
        if part.dynamic {
            out.push_str("{{ ");
            out.push_str(part.text.as_str());
            out.push_str(" }}");
        } else {
            out.push_str(part.text.as_str());
        }
    }
    out
}

/// Fold a consumed dropped-member gap into the preceding part's
/// authored range, so the recorded parts tile the merged span exactly.
/// A dropped tail always follows its condensed whitespace head (the
/// neighbour rules put a `Drop` between two kept text-family members in
/// no other position), so the fold target is always a static part.
fn fold_gap(parts: &mut StdVec<TextPart>, pending: &mut Option<u32>) {
    if let Some(gap_end) = pending.take()
        && let Some(last) = parts.last_mut()
    {
        debug_assert!(
            !last.dynamic,
            "a dropped run tail always follows its condensed head"
        );
        last.span.end = gap_end;
    }
}

/// Lower the maximal text/interpolation run starting at `start`,
/// returning the index just past it. Dropped whitespace lowers to
/// nothing (recorded); a one-node run lowers as the plain leaf; a
/// longer run merges.
pub(crate) fn lower_text_run<'a>(
    cx: &mut Cx<'a>,
    children: &[SurfaceChild<'a>],
    plan: &[TextAction<'a>],
    start: usize,
    out: &mut vize_s0::Vec<'a, Op<'a>>,
) -> usize {
    if let SurfaceChild::Text(token) = &children[start]
        && plan[start] == TextAction::Drop
    {
        let span = cx.token_span(token);
        cx.record(
            "condense.drop-whitespace",
            None,
            token.text,
            String::default(),
            span,
        );
        return start + 1;
    }

    // Scan the run: span-contiguous text/interpolation children whose
    // plan keeps them. Adjacent static members **fuse** into one part —
    // two list-adjacent text nodes are one DOM text run (the shape only
    // split or recovered S1 trees present; a parse emits maximal runs)
    // — and the condense collapse re-runs across the fused content, so
    // a whitespace run straddling the seam condenses exactly as the
    // one-node spelling does. A `Drop`-planned member inside the run (a
    // condensed whitespace run's tail) is consumed, recorded, and its
    // bytes folded into the preceding part's authored range, so the
    // parts still tile the merged span; dropped bytes with no following
    // member stay outside the unit.
    let mut parts: StdVec<TextPart> = StdVec::new();
    let mut members = 0usize;
    let mut i = start;
    let mut end = 0u32;
    let mut pending_gap: Option<u32> = None;
    while i < children.len() {
        let probe = pending_gap.unwrap_or(end);
        if i > start && !extends_run(cx, &children[i], probe) {
            break;
        }
        match &children[i] {
            SurfaceChild::Text(token) => {
                let span = cx.token_span(token);
                if plan[i] == TextAction::Drop {
                    // A condensed whitespace run's tail member: consume
                    // and record it here (never a run start — the
                    // pre-scan arm returns those).
                    cx.record(
                        "condense.drop-whitespace",
                        None,
                        token.text,
                        String::default(),
                        span,
                    );
                    pending_gap = Some(span.end);
                    i += 1;
                    continue;
                }
                let content = match plan[i] {
                    TextAction::Content(content) => content,
                    _ => token.text,
                };
                fold_gap(&mut parts, &mut pending_gap);
                match parts.last_mut() {
                    Some(last) if !last.dynamic => {
                        last.text.push_str(content);
                        last.span.end = span.end;
                    }
                    _ => parts.push(TextPart {
                        text: String::from(content),
                        span,
                        dynamic: false,
                    }),
                }
                end = span.end;
            }
            SurfaceChild::Interpolation(node) => {
                let span = Span::new(cx.offset(node.open.text), cx.token_span(&node.close).end);
                let (slice, _) = trimmed(cx, node.content.text);
                fold_gap(&mut parts, &mut pending_gap);
                parts.push(TextPart {
                    text: String::from(slice),
                    span,
                    dynamic: true,
                });
                end = span.end;
            }
            _ => break,
        }
        members += 1;
        i += 1;
    }
    if !cx.condense_suppressed() {
        for part in parts.iter_mut().filter(|part| !part.dynamic) {
            collapse_fused(&mut part.text);
        }
    }

    if members == 1 {
        // A lone node never merges (the legacy run grouping's own rule);
        // it lowers as the plain leaf, with the condensed content.
        match &children[start] {
            SurfaceChild::Text(token) => {
                let content = match plan[start] {
                    TextAction::Content(content) => content,
                    _ => token.text,
                };
                if content != token.text {
                    let span = cx.token_span(token);
                    cx.record(
                        "condense.whitespace",
                        None,
                        token.text,
                        String::from(content),
                        span,
                    );
                }
                super::leaf::lower_text(cx, token, content, out);
            }
            child => super::leaf::lower_leaf(cx, child, out),
        }
        return start + 1;
    }

    let span = Span::new(parts[0].span.start, end);
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
            cstr!("ui.text merged={members}"),
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
        // A mixed run is the compound representation: one
        // `ui.interpolation` under the pessimal opaque laws, parts
        // recorded beside the tree for the pass to validate and P2-11
        // to compile from.
        let rebuilt = rebuild_source(&parts);
        let mut source = StringBuilder::with_capacity_in(rebuilt.len(), cx.allocator);
        source.push_str(rebuilt.as_str());
        let expression = opaque_at(cx, OpaqueReason::Compound, source.into_str(), span);
        cx.record(
            "lower.compound",
            node,
            raw,
            cstr!(
                "ui.interpolation {} parts={} dynamic={}",
                desc(&expression),
                parts.len(),
                dynamic
            ),
            span,
        );
        cx.attach_texts(node, TextParts { parts });
        out.push(Op::Interpolation(Box::new_in(
            InterpolationOp { expression, span },
            &cx.allocator,
        )));
    }
    i
}
