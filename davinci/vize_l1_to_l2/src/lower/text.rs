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
//! (`crates/vize_atelier_core/src/steps/text.rs`) is exported but never
//! called by the shipped pipeline. The L2 port pulls both into the L1→L2 conversion,
//! for one decisive reason: **comments**. Both computations read comment
//! positions, and comments exist only in L1 unless the DOM
//! comment-preserving route asks the lowering to keep them as
//! `ui.comment` (the default lowering drops them under `drop.comment`).
//! Only the lowering still holds the information either computation
//! needs; a post-lowering pass would be comment-blind, and the shipped
//! answers are not.
//!
//! # What a comment does, per configuration
//!
//! **Preserved:** the comment is a real child — a non-text-like
//! neighbour for the remove-vs-condense rule and a hard boundary for run
//! merging.
//!
//! **Dropped (the shipped DOM default):** it is not a child at all.
//! Vue's parser builds no node for it, and its `onText` appends to the
//! previous child whenever that child is a text node, *without* a
//! contiguity check — so `a<!--c-->b` reaches whitespace condensing as
//! the single node `ab`, and `a<!--c-->\n<!--d-->b` as `a\nb`, which
//! condenses to `a b`. [`condense::plan_whitespace`](condense) therefore
//! absorbs a dropped comment into the text group around it and
//! [`run`](run) consumes it as a gap, so both halves see the child list
//! the shipped lane sees.
//!
//! An earlier reading had this the other way round — that a dropped
//! comment stays a run boundary, so `a<!--c-->b` "must stay two text
//! units". Measured against `@vue/compiler-dom` 3.5.41 and 3.6.0-beta.10
//! and against this workspace's own legacy lane, it does not: that
//! reading emitted `"a" + "b"` for `a<!--c-->b` and, worse,
//! `"a " + " b"` for `a <!--c--> b`, which renders one space too many.
//! `vize_atelier_dom/tests/dropped_comment_text_runs.rs` is the pin.
//! P2-5b's record
//! independently requires it: the position-classified opaque reasons —
//! [`OpaqueReason::Compound`] included — "are assignable only by the
//! L1→L2 lowering".
//!
//! # Condensing (Vue's `condense` strategy, the armature algorithm)
//!
//! Rule-for-rule the shipped `condense_whitespace`
//! (`whitespace.rs:69-165`), over the L1 child list of every lowered
//! region: leading/trailing whitespace-only text removed; a
//! whitespace-only run between two non-text-like neighbours with a
//! newline removed; spaces adjoining a comment and another comment or element
//! removed; condensed to one space otherwise; interior runs of
//! the Vue alphabet `[ \t\n\f\r]` in mixed text collapsed to one space.
//! Exemptions follow the shipped DOM configuration: `<pre>` subtrees
//! (`is_pre_tag`, `crates/vize_atelier_dom/src/compile/stage_options.rs`).
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
//! byte** (`vize_l2::expr::opaque`): a compound is never constant,
//! equal to nothing, and emittable only byte-verbatim-or-refusal — so
//! the rebuilt [`OpaqueExpr::source`] is a *display* form, and DOM
//! realization (P2-11) compiles from the recorded parts instead
//! ([`TextParts`], validated when attached to the lowered compound). The
//! rebuild spelling is canonical template syntax — static parts
//! verbatim, dynamic parts as `{{ <trimmed> }}` — with one shared rule
//! ([`rebuild_source`]). Merging never crosses a comment, a dropped node,
//! or any source gap: runs extend only over span-contiguous children, so
//! the merged span is the authored bytes exactly.
//!
//! [`OpaqueExpr::source`]: vize_l2::expr::OpaqueExpr::source

use vize_l2::op::Op;

mod branch_gap;
mod condense;
mod lane;
mod leading_newline;
mod run;

pub use branch_gap::{branch_gap_text, branch_gap_text_at};
pub use condense::TextAction;
pub(crate) use condense::suppresses_condense;
pub(crate) use lane::{Lane, TextWhitespace, block_offset, block_span};
pub use lane::{RunShape, TextLane, TextRun, TextSource};
pub(crate) use run::lower_text_run;
pub use vize_l2::analysis::text_facts::{TextPart, TextParts, rebuild_source};

#[cfg(debug_assertions)]
pub(crate) fn debug_assert_text_family_gaps(ops: &[Op<'_>]) {
    for pair in ops.windows(2) {
        let [prev, next] = pair else {
            continue;
        };
        let prev_end = match prev {
            Op::Text(text) => text.span.end,
            Op::Interpolation(interpolation) | Op::JsxChildValue(interpolation) => {
                interpolation.span.end
            }
            _ => continue,
        };
        let next_start = match next {
            Op::Text(text) => text.span.start,
            Op::Interpolation(interpolation) | Op::JsxChildValue(interpolation) => {
                interpolation.span.start
            }
            _ => continue,
        };
        debug_assert!(
            prev_end < next_start,
            "compound law broken: adjacent text-family ops touch at offset {prev_end} — mergeable adjacency must not survive the lowering",
        );
    }
}

#[cfg(not(debug_assertions))]
pub(crate) fn debug_assert_text_family_gaps(_: &[Op<'_>]) {}
