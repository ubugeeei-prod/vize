//! Project checked, source-ordered edits to the shared UTF-16 coordinate type.
use super::{EditError, EditSet, VersionedSource};
use vize_l0::line_index::{Position, Range};

impl<'edits, 'source, 'replacement, K: Copy + Eq> EditSet<'edits, 'source, 'replacement, K> {
    /// Recheck the exact current source and stream authored edit ranges in their
    /// existing order. One cursor spans the batch, including adjacent edits and
    /// repeated insertions at the same position. Replacements stay borrowed.
    pub fn positioned_edits(
        self,
        current: VersionedSource<'_, K>,
    ) -> Result<
        impl ExactSizeIterator<Item = (Range, &'replacement str)>
        + use<'edits, 'source, 'replacement, K>,
        EditError,
    > {
        self.check_current(current)?;
        let mut remaining = self.snapshot.root().source();
        let mut previous_end = 0;
        let mut position = Position::default();
        Ok(self.edits.iter().map(move |edit| {
            let span = edit.span();
            // Construction proves source order and both UTF-8 boundaries.
            let (gap, tail) = remaining.split_at((span.start - previous_end) as usize);
            let (replaced, tail) = tail.split_at((span.end - span.start) as usize);
            let start = position.after_text(gap);
            position = start.after_text(replaced);
            remaining = tail;
            previous_end = span.end;
            (Range::new(start, position), edit.replacement())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use vize_l0::{SourceRoot, Span, line_index::offset_to_position};

    #[test]
    #[expect(
        clippy::unwrap_used,
        reason = "directed source and edit boundaries are checked"
    )]
    fn full_ranges_match_single_shot_positions_at_every_valid_byte_boundary() {
        for source in ["", "x", "α😀\r\n\r\n雪\u{2028}z", "one\ntwo\rthree"] {
            let frame = VersionedSource::new("source", 8, SourceRoot::new(source).unwrap());
            let offsets: Vec<_> = (0..=source.len())
                .filter(|at| source.is_char_boundary(*at))
                .collect();
            let edits: Vec<_> = offsets
                .windows(2)
                .map(|offsets| {
                    let [start, end] = *offsets else {
                        unreachable!()
                    };
                    frame
                        .edit(Span::new(start as u32, end as u32), "x")
                        .unwrap()
                })
                .collect();
            let set = EditSet::new(frame, &edits).unwrap();
            let ranges = set.positioned_edits(frame).unwrap();
            assert_eq!(ranges.len(), edits.len());
            for ((range, replacement), edit) in ranges.zip(&edits) {
                assert_eq!(
                    range.start,
                    offset_to_position(source, edit.span.start as usize)
                );
                assert_eq!(
                    range.end,
                    offset_to_position(source, edit.span.end as usize)
                );
                assert_eq!(replacement, edit.replacement);
            }
            let stale = VersionedSource::new("source", 7, frame.root());
            assert_eq!(
                set.positioned_edits(stale).err(),
                Some(EditError::StaleVersion)
            );
        }
    }
}
