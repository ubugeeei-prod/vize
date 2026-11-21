//! Project checked, source-ordered edits to the shared UTF-16 coordinate type.
use super::{EditError, EditSet, SpanEdit, VersionedSource};
use vize_l0::{
    Span,
    line_index::{AsciiLineCursor, Position, Range},
};

// Keep the ASCII preflight bounded even for a single fix near the end of a large file.
const SHORT_PREFIX: usize = 256;

// Edit construction proves source order and both UTF-8 boundaries.
#[inline]
fn consume(source: &str, offset: u32, origin: Position, span: Span) -> (Range, &str) {
    let (prefix, tail) = source.split_at((span.end - offset) as usize);
    (
        Range::suffix_of(origin, prefix, (span.start - offset) as usize),
        tail,
    )
}

/// An edit set checked once against the current key, version and physical buffer.
pub struct PositionedEdits<'edits, 'source, 'replacement, K> {
    source: &'source str,
    edits: &'edits [SpanEdit<'source, 'replacement, K>],
}

impl<'edits, 'source, 'replacement, K: Copy + Eq> EditSet<'edits, 'source, 'replacement, K> {
    /// Bind source-ordered edits to the exact current buffer before publication.
    #[inline]
    pub fn positioned_edits(
        self,
        current: VersionedSource<'_, K>,
    ) -> Result<PositionedEdits<'edits, 'source, 'replacement, K>, EditError> {
        self.check_current(current)?;
        Ok(PositionedEdits {
            source: self.snapshot.root().source(),
            edits: self.edits,
        })
    }
}

impl<'edits, 'source, 'replacement, K: Copy + Eq>
    PositionedEdits<'edits, 'source, 'replacement, K>
{
    #[must_use]
    pub fn edits(&self) -> &'edits [SpanEdit<'source, 'replacement, K>] {
        self.edits
    }

    /// Locate both endpoints directly when the validated selection contains one edit.
    #[inline]
    pub fn single(&self) -> Option<(Range, &'replacement str)> {
        let [edit] = self.edits else {
            return None;
        };
        let span = edit.span();
        let (prefix, _) = self.source.split_at(span.end as usize);
        let range = if (32..=SHORT_PREFIX).contains(&prefix.len())
            && let Some(mut lines) = AsciiLineCursor::new(prefix)
        {
            Range::new(lines.position(span.start), lines.position(span.end))
        } else {
            Range::suffix_of(Position::default(), prefix, span.start as usize)
        };
        Some((range, edit.replacement()))
    }

    /// Locate ordered edits with shared line positions and borrowed replacements.
    pub fn ranges(
        self,
    ) -> impl ExactSizeIterator<Item = (Range, &'replacement str)> + use<'edits, 'source, 'replacement, K>
    {
        let mut remaining = self.source;
        let end = self.edits.last().map_or(0, |edit| edit.span.end as usize);
        let (prefix, _) = remaining.split_at(end);
        // A bounded prefix amortizes one ASCII check across nearby edits.
        // Sparse edits in larger files skip whole lines with suffix_of.
        let mut ascii = if end <= SHORT_PREFIX {
            AsciiLineCursor::new(prefix)
        } else {
            None
        };
        let mut previous_end = 0;
        let mut position = Position::default();
        self.edits.iter().map(move |edit| {
            let span = edit.span();
            if let Some(lines) = &mut ascii {
                let range = Range::new(lines.position(span.start), lines.position(span.end));
                return (range, edit.replacement());
            }
            let (range, tail) = consume(remaining, previous_end, position, span);
            position = range.end;
            remaining = tail;
            previous_end = span.end;
            (range, edit.replacement())
        })
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
        let long_ascii = "line\r\n".repeat(48);
        let long_unicode = "雪😀\n".repeat(48);
        for source in [
            "",
            "x",
            "α😀\r\n\r\n雪\u{2028}z",
            "one\ntwo\rthree",
            &long_ascii,
            &long_unicode,
        ] {
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
            let ranges = set.positioned_edits(frame).unwrap().ranges();
            assert_eq!(ranges.len(), edits.len());
            for ((range, replacement), edit) in ranges.zip(&edits) {
                let one = EditSet::new(frame, core::slice::from_ref(edit)).unwrap();
                assert_eq!(
                    one.positioned_edits(frame).unwrap().single(),
                    Some((range, replacement))
                );
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
