//! The run scans: what one text-family run lowers to, from the plan.

use alloc::vec::Vec as StdVec;

use vize_l0::{Span, String};
use vize_l1::markup::SurfaceChild;

use super::super::condense::{collapse_fused, extends_run};
use super::super::leading_newline::normalize_special_text;
use super::super::{TextAction, TextPart};
use super::{Consumed, Lane, RunShape, TextRun};
use crate::lower::leaf::keeps_directive_comment;

impl<'a> Lane<'a> {
    pub(super) fn text_run(
        &self,
        children: &[SurfaceChild<'a>],
        plan: &[TextAction<'a>],
        start: usize,
    ) -> TextRun<'a> {
        let mut consumed = StdVec::new();
        let Some(first) = children.get(start) else {
            return TextRun {
                end: start + 1,
                shape: RunShape::Dropped,
                consumed,
            };
        };
        // The plan holds one action per child; a missing one lowers as authored.
        let action_at = |index: usize| plan.get(index).copied().unwrap_or(TextAction::Keep);
        if let SurfaceChild::Text(token) = first
            && action_at(start) == TextAction::Drop
        {
            consumed.push(Consumed {
                kind: "condense.drop-whitespace",
                text: token.text,
                span: self.token_span(token),
            });
            return TextRun {
                end: start + 1,
                shape: RunShape::Dropped,
                consumed,
            };
        }

        // A final child cannot form a compound. Keep its borrowed leaf instead
        // of constructing owned parts that the one-member arm would discard.
        if start + 1 == children.len() {
            return TextRun {
                end: start + 1,
                shape: self.leaf(first, action_at(start)),
                consumed,
            };
        }

        // Scan the run: span-contiguous text/interpolation children whose
        // plan keeps them. Adjacent static members **fuse** into one part —
        // two list-adjacent text nodes are one DOM text run (the shape only
        // split or recovered L1 trees present; a parse emits maximal runs)
        // — and the condense collapse re-runs across the fused content, so
        // a whitespace run straddling the seam condenses exactly as the
        // one-node spelling does. A `Drop`-planned member inside the run (a
        // condensed whitespace run's tail) is consumed, recorded, and its
        // bytes folded into the preceding part's authored range, so the
        // parts still tile the merged span; dropped bytes with no following
        // member stay outside the unit.
        let mut parts: StdVec<TextPart> = StdVec::new();
        let mut needs_collapse = false;
        let mut members = 0usize;
        let mut i = start;
        let mut end = 0u32;
        let mut pending_gap: Option<u32> = None;
        while let Some(child) = children.get(i) {
            let probe = pending_gap.unwrap_or(end);
            // A comment the compile is not preserving is not a run boundary:
            // it is not a child at all. Vue's parser builds no node for it,
            // and `onText` appends the next chunk to the previous text child
            // without checking contiguity — so `a<!--c-->b` reaches
            // condensing as the one node `ab`. Consumed here exactly like a
            // dropped whitespace member: recorded, and its bytes folded into
            // the preceding part's range so the parts still tile the merged
            // span. Contiguity is still required, so the merged span is
            // still the authored bytes.
            if i > start
                && let SurfaceChild::Comment(token) = child
                && !self.preserve_comments()
                && !keeps_directive_comment(token.text)
                && token.leading.is_empty()
                && self.offset(token.text) == probe
            {
                let span = self.token_span(token);
                consumed.push(Consumed {
                    kind: "drop.comment",
                    text: token.text,
                    span,
                });
                pending_gap = Some(span.end);
                i += 1;
                continue;
            }
            if i > start && !extends_run(self, child, probe) {
                break;
            }
            match child {
                SurfaceChild::Text(token) => {
                    let span = self.token_span(token);
                    let action = action_at(i);
                    if action == TextAction::Drop {
                        // A condensed whitespace run's tail member: consume
                        // and record it here (never a run start — the
                        // pre-scan arm returns those).
                        consumed.push(Consumed {
                            kind: "condense.drop-whitespace",
                            text: token.text,
                            span,
                        });
                        pending_gap = Some(span.end);
                        i += 1;
                        continue;
                    }
                    let content = match action {
                        TextAction::Content(content) => content,
                        _ => {
                            needs_collapse = true;
                            token.text
                        }
                    };
                    fold_gap(&mut parts, &mut pending_gap);
                    match parts.last_mut() {
                        Some(last) if !last.dynamic => {
                            needs_collapse = true;
                            last.text.push_str(content);
                            last.span.end = span.end;
                        }
                        _ => parts.push(TextPart {
                            text: String::from(content),
                            span,
                            dynamic: false,
                            raw_html: false,
                        }),
                    }
                    end = span.end;
                }
                SurfaceChild::Interpolation(node) => {
                    let span = Span::new(
                        self.offset(node.open.text),
                        self.token_span(&node.close).end,
                    );
                    let slice = node.content.text.trim();
                    fold_gap(&mut parts, &mut pending_gap);
                    parts.push(TextPart {
                        text: String::from(slice),
                        span,
                        dynamic: true,
                        raw_html: node.is_raw_html(),
                    });
                    end = span.end;
                }
                _ => break,
            }
            members += 1;
            i += 1;
        }
        if members == 1 {
            // A lone node never merges (the legacy run grouping's own rule);
            // it lowers as the plain leaf, with the condensed content.
            // `i` is past everything the scan consumed, not just the lone
            // member: a dropped whitespace tail or a dropped comment was
            // already recorded, and returning `start + 1` would let the
            // structural walker record it a second time.
            return TextRun {
                end: i,
                shape: self.leaf(first, action_at(start)),
                consumed,
            };
        }
        // Planned Content is normalized already. Raw members and static fusions
        // still require the original collapse, including whitespace across seams.
        if needs_collapse && !self.condense_suppressed() {
            for part in parts.iter_mut().filter(|part| !part.dynamic) {
                collapse_fused(&mut part.text);
            }
        }
        if self.ignore_newline_at().is_some() || self.normalize_pre_newlines() {
            for part in parts.iter_mut().filter(|part| !part.dynamic) {
                if let Some(content) =
                    normalize_special_text(self, part.text.as_str(), part.span.start)
                {
                    part.text = content;
                }
            }
        }
        let span = Span::new(parts.first().map_or(end, |part| part.span.start), end);
        TextRun {
            end: i,
            shape: RunShape::Merged {
                parts,
                span,
                members,
            },
            consumed,
        }
    }

    /// A contiguous text/interpolation run under `v-pre`. Interpolation
    /// delimiters render as authored text, while Vue's normal whitespace
    /// condense still applies unless an enclosing `<pre>` disabled it.
    pub(super) fn frozen_run(
        &self,
        children: &[SurfaceChild<'a>],
        plan: &[TextAction<'a>],
        start: usize,
    ) -> TextRun<'a> {
        let mut consumed = StdVec::new();
        let first = children.get(start);
        let Some(mut end) = first.and_then(|first| self.family_end(first)) else {
            return TextRun {
                end: start + 1,
                shape: RunShape::Dropped,
                consumed,
            };
        };
        let start_offset = first
            .and_then(|first| self.family_start(first))
            .unwrap_or(end);
        let mut i = start + 1;
        while let Some(child) = children.get(i) {
            let Some(next_start) = self.family_start(child) else {
                break;
            };
            if next_start != end {
                break;
            }
            let Some(next_end) = self.family_end(child) else {
                break;
            };
            end = next_end;
            i += 1;
        }
        if i == start + 1
            && let Some(SurfaceChild::Text(token)) = first
            && plan.get(start) == Some(&TextAction::Drop)
        {
            consumed.push(Consumed {
                kind: "condense.drop-whitespace",
                text: token.text,
                span: self.token_span(token),
            });
            return TextRun {
                end: i,
                shape: RunShape::Dropped,
                consumed,
            };
        }

        let span = Span::new(start_offset, end);
        let raw = self
            .src
            .block
            .root_source()
            .get(span.start as usize..span.end as usize)
            .unwrap_or_default();
        let mut content = String::from(raw);
        if let Some(normalized) = normalize_special_text(self, content.as_str(), start_offset) {
            content = normalized;
        }
        if !self.condense_suppressed() {
            collapse_fused(&mut content);
        }
        TextRun {
            end: i,
            shape: RunShape::Frozen { content, span },
            consumed,
        }
    }

    fn family_start(&self, child: &SurfaceChild<'_>) -> Option<u32> {
        match child {
            SurfaceChild::Text(token) => Some(self.offset(token.text)),
            SurfaceChild::Interpolation(node) => Some(self.offset(node.open.text)),
            _ => None,
        }
    }

    fn family_end(&self, child: &SurfaceChild<'_>) -> Option<u32> {
        match child {
            SurfaceChild::Text(token) => Some(self.token_span(token).end),
            SurfaceChild::Interpolation(node) => Some(self.token_span(&node.close).end),
            _ => None,
        }
    }
}

/// Fold a consumed dropped-member gap into the preceding part's
/// authored range, so the recorded parts tile the merged span exactly.
///
/// The gap is either a condensed whitespace run's dropped tail — which
/// the neighbour rules only ever place after its condensed head, a
/// static part — or a comment this compile is not preserving, which can
/// sit after either kind. Both fold the same way: the preceding part's
/// range grows to cover the consumed bytes.
fn fold_gap(parts: &mut StdVec<TextPart>, pending: &mut Option<u32>) {
    if let Some(gap_end) = pending.take()
        && let Some(last) = parts.last_mut()
    {
        last.span.end = gap_end;
    }
}
