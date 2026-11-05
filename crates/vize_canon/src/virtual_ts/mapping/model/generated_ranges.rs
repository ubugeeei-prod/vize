//! Generated coordinate transforms shared by core and supplementary links.

use super::{ProjectionMapping, VizeMapping, add_delta, shift_both, shift_generated_range};
use crate::virtual_ts::VizeSubSpan;
use std::ops::Range;

impl ProjectionMapping {
    /// Move generated ranges after replacing `old_len` bytes at `start`.
    pub fn note_generated_replacement(&mut self, start: usize, old_len: usize, new_len: usize) {
        let old_end = start.saturating_add(old_len);
        let delta = new_len as isize - old_len as isize;
        if delta == 0 {
            return;
        }
        // An edit inside a byte-for-byte row changes the distance from its
        // start for every following byte. Keep the unchanged prefix and
        // suffix as separate, aligned rows. Most edits fall between rows;
        // avoid rebuilding the mapping for those.
        let needs_split = |span: &VizeMapping| {
            span.sub_spans.is_empty()
                && span.gen_range.start < start
                && old_end < span.gen_range.end
                && span.gen_range.end - span.gen_range.start
                    == span.src_range.end - span.src_range.start
        };
        if !self.spans.iter().any(needs_split) {
            for span in &mut self.spans {
                shift_generated_range(&mut span.gen_range, start, old_end, delta);
                replace_subspans(&mut span.sub_spans, start, old_end, new_len, delta);
            }
            for link in self
                .semantic_links
                .iter_mut()
                .chain(&mut self.prop_default_key_links)
            {
                shift_generated_range(&mut link.source_range, start, old_end, delta);
                shift_generated_range(&mut link.target_range, start, old_end, delta);
            }
            return;
        }
        let spans = core::mem::take(&mut self.spans);
        let meta = core::mem::take(&mut self.meta);
        self.authored_disjoint = true;
        for (index, mut span) in spans.into_iter().enumerate() {
            let row_meta = meta.get(index).copied().unwrap_or_default();
            if needs_split(&span) {
                let source_start = span.src_range.start + start - span.gen_range.start;
                let source_end = source_start + old_len;
                self.push_with(
                    VizeMapping::new(
                        span.gen_range.start..start,
                        span.src_range.start..source_start,
                    ),
                    row_meta,
                );
                if old_len > 0 {
                    self.push_with(
                        VizeMapping::new(start..start + new_len, source_start..source_end),
                        row_meta,
                    );
                }
                self.push_with(
                    VizeMapping::new(
                        add_delta(old_end, delta)..add_delta(span.gen_range.end, delta),
                        source_end..span.src_range.end,
                    ),
                    row_meta,
                );
            } else {
                shift_generated_range(&mut span.gen_range, start, old_end, delta);
                replace_subspans(&mut span.sub_spans, start, old_end, new_len, delta);
                self.push_with(span, row_meta);
            }
        }
        for link in self
            .semantic_links
            .iter_mut()
            .chain(&mut self.prop_default_key_links)
        {
            shift_generated_range(&mut link.source_range, start, old_end, delta);
            shift_generated_range(&mut link.target_range, start, old_end, delta);
        }
    }

    /// Move rows when `void (expr)` becomes `const __expr_N = expr`.
    pub fn retarget_expression_binding(
        &mut self,
        stmt: Range<usize>,
        expr: Range<usize>,
        prefix_delta: isize,
        total_delta: isize,
    ) {
        let adjust = |range: &mut Range<usize>| {
            if range.start >= stmt.end {
                shift_both(range, total_delta);
            } else if range.start >= expr.start && range.end <= expr.end {
                shift_both(range, prefix_delta);
            } else if range.end > stmt.end && range.start < stmt.end {
                range.end = add_delta(range.end, total_delta);
            }
        };
        for span in &mut self.spans {
            adjust(&mut span.gen_range);
            for sub in &mut span.sub_spans {
                adjust(&mut sub.gen_range);
            }
        }
        for link in self
            .semantic_links
            .iter_mut()
            .chain(&mut self.prop_default_key_links)
        {
            adjust(&mut link.source_range);
            adjust(&mut link.target_range);
        }
    }
}

fn replace_subspans(
    spans: &mut Vec<VizeSubSpan>,
    start: usize,
    old_end: usize,
    new_len: usize,
    delta: isize,
) {
    let split = |span: &VizeSubSpan| {
        span.gen_range.len() == span.src_range.len()
            && span.gen_range.start < start
            && old_end < span.gen_range.end
    };
    if !spans.iter().any(split) {
        for span in spans {
            shift_generated_range(&mut span.gen_range, start, old_end, delta);
        }
        return;
    }
    let original = core::mem::take(spans);
    for mut span in original {
        if split(&span) {
            let source_start = span.src_range.start + start - span.gen_range.start;
            let source_end = source_start + old_end - start;
            spans.push(VizeSubSpan {
                gen_range: span.gen_range.start..start,
                src_range: span.src_range.start..source_start,
            });
            if old_end > start {
                spans.push(VizeSubSpan {
                    gen_range: start..start + new_len,
                    src_range: source_start..source_end,
                });
            }
            spans.push(VizeSubSpan {
                gen_range: add_delta(old_end, delta)..add_delta(span.gen_range.end, delta),
                src_range: source_end..span.src_range.end,
            });
        } else {
            shift_generated_range(&mut span.gen_range, start, old_end, delta);
            spans.push(span);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_in_an_identity_subspan_preserves_both_authored_sides() {
        let prefix = VizeSubSpan {
            gen_range: 0..4,
            src_range: 0..4,
        };
        let mut mapping = ProjectionMapping::from_spans(vec![VizeMapping {
            gen_range: 0..11,
            src_range: 0..10,
            sub_spans: vec![
                prefix.clone(),
                VizeSubSpan {
                    gen_range: 5..11,
                    src_range: 4..10,
                },
            ],
        }]);
        mapping.note_generated_replacement(8, 0, 3);
        assert_eq!(
            mapping.spans(),
            &[VizeMapping {
                gen_range: 0..14,
                src_range: 0..10,
                sub_spans: vec![
                    prefix,
                    VizeSubSpan {
                        gen_range: 5..8,
                        src_range: 4..7,
                    },
                    VizeSubSpan {
                        gen_range: 11..14,
                        src_range: 7..10,
                    },
                ],
            }],
        );
        assert_eq!(mapping.diagnostic_range_to_authored(6, 7), Some((5, 6)));
        assert_eq!(mapping.diagnostic_range_to_authored(12, 13), Some((8, 9)));
    }

    #[test]
    fn replacement_in_a_nonlinear_subspan_retains_its_original_authored_range() {
        let mut mapping = ProjectionMapping::from_spans(vec![VizeMapping {
            gen_range: 0..12,
            src_range: 0..8,
            sub_spans: vec![VizeSubSpan {
                gen_range: 2..10,
                src_range: 2..6,
            }],
        }]);
        mapping.note_generated_replacement(5, 1, 3);
        assert_eq!(mapping.spans()[0].gen_range, 0..14);
        assert_eq!(mapping.spans()[0].src_range, 0..8);
        assert_eq!(
            mapping.spans()[0].sub_spans,
            vec![VizeSubSpan {
                gen_range: 2..12,
                src_range: 2..6,
            }],
        );
    }
}
