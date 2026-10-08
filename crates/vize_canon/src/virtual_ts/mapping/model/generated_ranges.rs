//! Generated coordinate transforms shared by core and supplementary links.

use super::{ProjectionMapping, VizeMapping, add_delta, shift_both, shift_generated_range};
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
                for sub in &mut span.sub_spans {
                    shift_generated_range(&mut sub.gen_range, start, old_end, delta);
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
                for sub in &mut span.sub_spans {
                    shift_generated_range(&mut sub.gen_range, start, old_end, delta);
                }
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
