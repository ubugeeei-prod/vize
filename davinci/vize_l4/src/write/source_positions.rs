//! The line and UTF-16 column of every recorded source-map segment, resolved
//! on the bytes of the generated code and of the template source.
//!
//! A line starts after every JavaScript line terminator (LF, CR, LS, PS, and
//! CR LF once). The source side looks its offset up in a table of line
//! starts; the generated side scans forward once, since the caller sorts the
//! segments by generated offset. A column counts the UTF-16 code units of
//! the scalars between the line start and the offset, one per lead byte plus
//! one more per four-byte lead, and an offset inside a scalar moves back to
//! its start. Creusot proves each contract for texts of every length
//! (`creusot.rs` beside this file states the line starts and the lemmas,
//! and `tests/formal/creusot/unicode` the UTF-8 facts).

use super::line_endings::line_break_len;
use alloc::vec::Vec;

#[cfg(creusot)]
use crate::{
    a_line_lies_between_its_starts, a_scan_step, a_split_crlf_stays_on_its_line,
    a_start_follows_its_breaks, floor, floor_is_the_greatest_boundary, is_line_start, js_table,
    last_start, line_breaks, line_breaks_fit, line_starts_are_boundaries, not_inside,
    terminator_width, the_end_is_its_own_floor, the_floor_grows, the_floor_skips_trailing_bytes,
    the_last_start_is_a_line_start, the_line_starts_increase,
};
#[cfg(creusot)]
use creusot_std::prelude::{Clone, *};
#[cfg(creusot)]
use davinci_unicode::{
    a_later_boundary_follows, ascii_bytes_weigh_one_each, boundaries_are_the_lead_bytes, boundary,
    is_trail, lead_bytes_count_the_units, lead_weight, units, units_fit, utf8_from, weights,
    weights_grow,
};

/// A recorded mapping anchor, as raw byte offsets.
///
/// `generated_offset` is a byte offset into the code buffer captured at the
/// moment the mapped token was about to be written; `source_offset` is the byte
/// offset of the originating node in the original template source. Both are
/// resolved to (line, column) at `SourceMapBuilder::finish`. `name` is an
/// index into the builder's `names` table when the anchor carries a known
/// source symbol (a prop key, a static attribute name), or `None` for anonymous
/// anchors; it becomes the optional 5th VLQ field of the segment.
#[cfg_attr(not(creusot), derive(Debug, Clone, Copy))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub(super) struct Segment {
    pub(super) generated_offset: u32,
    pub(super) source_offset: u32,
    pub(super) name: Option<u32>,
}

/// A segment whose endpoints have been resolved to 0-indexed line/column.
#[cfg_attr(not(creusot), derive(Debug, Clone, Copy))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub(super) struct ResolvedSegment {
    pub(super) generated_line: u32,
    pub(super) generated_column: u32,
    pub(super) source_line: u32,
    pub(super) source_column: u32,
    /// Index into the v3 `names` array when this anchor carries a known source
    /// symbol, carried through unchanged from the recorded segment.
    pub(super) name: Option<u32>,
}

/// Resolve every segment's generated and source byte offsets to 0-indexed
/// (line, column) pairs.
///
/// The generated side uses a single monotonic forward scan (segments are sorted
/// by generated offset by the caller). The source side uses a precomputed table
/// of line-start byte offsets plus binary search, since segments are not sorted
/// by source offset.
///
/// Columns are counted in UTF-16 code units to match what source-map consumers
/// (browsers, the `source-map` npm package) expect. Both the generated render
/// code and template identifiers are overwhelmingly ASCII, so this equals the
/// byte/char column in the common case and only diverges on embedded non-ASCII.
#[cfg_attr(creusot, requires(utf8_from(code@, 0) && utf8_from(source@, 0)))]
#[cfg_attr(creusot, requires(code@.len() <= u32::MAX@ && source@.len() <= u32::MAX@))]
#[cfg_attr(creusot, requires(forall<i: Int, j: Int> 0 <= i && i <= j && j < segments@.len()
    ==> segments@[i].generated_offset@ <= segments@[j].generated_offset@))]
#[cfg_attr(creusot, ensures(result@.len() == segments@.len()))]
#[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < segments@.len() ==> {
    let generated = floor(code@, segments@[i].generated_offset@);
    let authored = floor(source@, segments@[i].source_offset@);
    result@[i].generated_line@ == line_breaks(code@, generated)
        && result@[i].generated_column@ == units(code@, last_start(code@, generated), generated)
        && result@[i].source_line@ == line_breaks(source@, authored)
        && result@[i].source_column@ == units(source@, last_start(source@, authored), authored)
        && result@[i].name == segments@[i].name
}))]
pub(super) fn resolve_positions(
    code: &[u8],
    source: &[u8],
    segments: &[Segment],
) -> Vec<ResolvedSegment> {
    let source_line_starts = line_start_table(source);

    let mut resolved: Vec<ResolvedSegment> = Vec::with_capacity(segments.len());

    let mut cursor = 0usize; // byte index into the generated code
    let mut gen_line = 0u32;
    let mut gen_line_start = 0usize;

    #[cfg_attr(creusot, invariant(resolved@.len() == produced.len()))]
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < produced.len() ==> {
        let generated = floor(code@, segments@[i].generated_offset@);
        let authored = floor(source@, segments@[i].source_offset@);
        resolved@[i].generated_line@ == line_breaks(code@, generated)
            && resolved@[i].generated_column@ == units(code@, last_start(code@, generated), generated)
            && resolved@[i].source_line@ == line_breaks(source@, authored)
            && resolved@[i].source_column@ == units(source@, last_start(source@, authored), authored)
            && resolved@[i].name == segments@[i].name
    }))]
    #[cfg_attr(creusot, invariant(cursor@ <= code@.len() && not_inside(code@, cursor@)))]
    #[cfg_attr(creusot, invariant(gen_line@ == line_breaks(code@, cursor@)))]
    #[cfg_attr(creusot, invariant(gen_line_start@ == last_start(code@, cursor@)))]
    #[cfg_attr(creusot, invariant(forall<k: Int> produced.len() <= k && k < segments@.len()
        ==> cursor@ <= floor(code@, segments@[k].generated_offset@)))]
    for seg in segments {
        // Generated side: advance the forward scan to this segment's offset.
        #[cfg(creusot)]
        proof_assert!(*seg == segments@[produced.len() - 1]);
        let target = floor_char_boundary(code, seg.generated_offset as usize);
        #[cfg(creusot)]
        proof_assert!(target@ == floor(code@, segments@[produced.len() - 1].generated_offset@));
        #[cfg(creusot)]
        proof_assert!(cursor@ <= target@);
        #[cfg_attr(creusot, invariant(cursor@ <= target@ && not_inside(code@, cursor@)))]
        #[cfg_attr(creusot, invariant(gen_line@ == line_breaks(code@, cursor@)))]
        #[cfg_attr(creusot, invariant(gen_line_start@ == last_start(code@, cursor@)))]
        while cursor < target {
            #[cfg(creusot)]
            proof_assert!(a_scan_step(code@, cursor@); a_split_crlf_stays_on_its_line(code@, cursor@); true);
            let width = line_break_len(code, cursor);
            if width > 0 && cursor + width <= target {
                cursor += width;
                #[cfg(creusot)]
                proof_assert!(line_breaks_fit(code@, cursor@); true);
                gen_line += 1;
                gen_line_start = cursor;
            } else {
                #[cfg(creusot)]
                proof_assert!(target@ < code@.len() ==> { boundaries_are_the_lead_bytes(code@, 0, target@); !is_trail(code@[target@]) });
                cursor += 1;
            }
        }
        #[cfg(creusot)]
        proof_assert!(cursor@ == target@);
        #[cfg(creusot)]
        proof_assert!(the_last_start_is_a_line_start(code@, cursor@);
            line_starts_are_boundaries(code@, gen_line_start@);
            a_later_boundary_follows(code@, 0, gen_line_start@, target@);
            true);
        let generated_column = utf16_len(code, gen_line_start, target);

        // Source side: binary search the line-start table.
        let (source_line, source_column) =
            resolve_in_table(source, &source_line_starts, seg.source_offset as usize);

        resolved.push(ResolvedSegment {
            generated_line: gen_line,
            generated_column,
            source_line,
            source_column,
            name: seg.name,
        });
        #[cfg(creusot)]
        proof_assert!(forall<k: Int> produced.len() <= k && k < segments@.len()
            ==> { the_floor_grows(code@, segments@[produced.len() - 1].generated_offset@, segments@[k].generated_offset@);
                cursor@ <= floor(code@, segments@[k].generated_offset@) });
    }

    resolved
}

/// Byte offsets at which each line of `text` begins. Always starts with `0`;
/// each JavaScript line terminator adds the offset just past it.
#[cfg_attr(creusot, ensures(js_table(text@, result@)))]
pub(super) fn line_start_table(text: &[u8]) -> Vec<usize> {
    let mut starts = Vec::with_capacity(16);
    starts.push(0);
    let mut cursor = 0;
    #[cfg(creusot)]
    proof_assert!(starts@.len() == 1 && line_breaks(text@, 0) == 0);
    #[cfg_attr(creusot, invariant(cursor@ <= text@.len() && not_inside(text@, cursor@)))]
    #[cfg_attr(creusot, invariant(starts@.len() == line_breaks(text@, cursor@) + 1 && starts@[0]@ == 0))]
    #[cfg_attr(creusot, invariant(forall<k: Int> 1 <= k && k < starts@.len()
        ==> 1 <= starts@[k]@ && starts@[k]@ <= cursor@
            && line_breaks(text@, starts@[k]@) == k && is_line_start(text@, starts@[k]@)))]
    while cursor < text.len() {
        #[cfg(creusot)]
        proof_assert!(line_breaks_fit(text@, cursor@); line_breaks(text@, cursor@) >= 0);
        #[cfg(creusot)]
        proof_assert!(a_scan_step(text@, cursor@); {
            let width = terminator_width(text@, cursor@);
            (width > 0 ==> is_line_start(text@, cursor@ + width)
                && line_breaks(text@, cursor@ + width) == line_breaks(text@, cursor@) + 1
                && not_inside(text@, cursor@ + width) && cursor@ + width <= text@.len())
                && (width == 0 ==> line_breaks(text@, cursor@ + 1) == line_breaks(text@, cursor@)
                    && not_inside(text@, cursor@ + 1))
        });
        let width = line_break_len(text, cursor);
        cursor += width.max(1);
        if width > 0 {
            #[cfg(creusot)]
            let before = snapshot!(starts@);
            starts.push(cursor);
            #[cfg(creusot)]
            proof_assert!(starts@.len() == before.len() + 1 && starts@[before.len()] == cursor
                && forall<k: Int> 0 <= k && k < before.len() ==> starts@[k] == before[k]);
        }
    }
    starts
}

/// Resolve a byte `offset` into `text` to a 0-indexed (line, column) using a
/// precomputed `line_starts` table. Column is in UTF-16 code units.
#[cfg_attr(creusot, requires(utf8_from(text@, 0) && text@.len() <= u32::MAX@))]
#[cfg_attr(creusot, requires(js_table(text@, line_starts@)))]
#[cfg_attr(creusot, ensures({
    let at = floor(text@, offset@);
    result.0@ == line_breaks(text@, at) && result.1@ == units(text@, last_start(text@, at), at)
}))]
pub(super) fn resolve_in_table(text: &[u8], line_starts: &[usize], offset: usize) -> (u32, u32) {
    let offset = floor_char_boundary(text, offset);
    #[cfg(creusot)]
    proof_assert!(forall<i: Int, j: Int> 0 <= i && i < j && j < line_starts@.len()
        ==> { the_line_starts_increase(text@, line_starts@, i, j); line_starts@[i]@ < line_starts@[j]@ });
    #[cfg(creusot)]
    proof_assert!(line_starts.deep_model().sorted());
    // The line is the index of the greatest line-start <= offset.
    let line = match line_starts.binary_search(&offset) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1), // i >= 1 because line_starts[0] == 0 <= offset
    };
    let line_start = match line_starts.get(line) {
        Some(&start) => start,
        None => 0,
    };
    #[cfg(creusot)]
    proof_assert!(a_line_lies_between_its_starts(text@, line_starts@, line@, offset@); true);
    #[cfg(creusot)]
    proof_assert!(a_start_follows_its_breaks(text@, line_starts@, line@);
        line_starts_are_boundaries(text@, line_start@);
        a_later_boundary_follows(text@, 0, line_start@, offset@);
        line_breaks_fit(text@, offset@);
        true);
    let column = utf16_len(text, line_start, offset);
    (line as u32, column)
}

/// The greatest scalar boundary at or before `offset`, clamped to the text.
#[cfg_attr(creusot, requires(utf8_from(text@, 0)))]
#[cfg_attr(creusot, ensures(result@ == floor(text@, offset@)))]
#[cfg_attr(creusot, ensures(result@ <= text@.len() && boundary(text@, 0, result@)))]
#[expect(
    clippy::manual_range_contains,
    reason = "Creusot 0.14 has no contract for Range::contains"
)]
pub(super) fn floor_char_boundary(text: &[u8], offset: usize) -> usize {
    #[cfg(creusot)]
    proof_assert!(floor_is_the_greatest_boundary(text@, offset@); the_end_is_its_own_floor(text@); true);
    #[cfg(creusot)]
    let asked = snapshot!(offset);
    let mut offset = offset.min(text.len());
    #[cfg_attr(creusot, invariant(offset@ <= text@.len()))]
    #[cfg_attr(creusot, invariant(floor(text@, offset@) == floor(text@, (*asked)@)))]
    while offset > 0 {
        #[cfg(creusot)]
        proof_assert!(offset@ < text@.len() ==> { the_floor_skips_trailing_bytes(text@, offset@); true });
        match text.get(offset) {
            Some(&byte) if byte >= 0x80 && byte < 0xc0 => offset -= 1,
            _ => break,
        }
    }
    offset
}

/// UTF-16 code units of the scalars that begin in `text[from..to]`.
#[cfg_attr(creusot, requires(utf8_from(text@, from@) && from@ <= to@ && to@ <= text@.len()))]
#[cfg_attr(creusot, requires(text@.len() <= u32::MAX@))]
#[cfg_attr(creusot, ensures(result@ == units(text@, from@, to@)))]
#[expect(
    clippy::manual_range_contains,
    reason = "Creusot 0.14 has no contract for Range::contains"
)]
#[inline]
pub(super) fn utf16_len(text: &[u8], from: usize, to: usize) -> u32 {
    let Some(part) = text.get(from..to) else {
        return 0;
    };
    #[cfg(creusot)]
    proof_assert!(lead_bytes_count_the_units(text@, from@, to@); units_fit(text@, from@, to@);
        weights(text@, from@, to@) == units(text@, from@, to@)
            && units(text@, from@, to@) <= text@.len() - from@);
    #[cfg(creusot)]
    proof_assert!(forall<i: Int> from@ <= i && i < to@ ==> text@[i] == part@[i - from@]);
    let mut top = 0;
    #[cfg_attr(creusot, invariant(
        (top@ < 128) == forall<i: Int> 0 <= i && i < produced.len() ==> part@[i]@ < 128
    ))]
    for &byte in part {
        #[cfg(creusot)]
        proof_assert!(byte == part@[produced.len() - 1]);
        if byte > top {
            top = byte;
        }
    }
    if top < 0x80 {
        #[cfg(creusot)]
        proof_assert!(ascii_bytes_weigh_one_each(text@, from@, to@);
            weights(text@, from@, to@) == to@ - from@);
        return part.len() as u32;
    }
    let mut n = 0u32;
    #[cfg_attr(creusot, invariant(n@ == weights(text@, from@, from@ + produced.len())))]
    for &byte in part {
        #[cfg(creusot)]
        proof_assert!(byte == part@[produced.len() - 1]);
        #[cfg(creusot)]
        proof_assert!(byte == text@[from@ + produced.len() - 1]);
        #[cfg(creusot)]
        proof_assert!(weights_grow(text@, from@, from@ + produced.len(), to@);
            weights(text@, from@, from@ + produced.len()) <= text@.len() - from@);
        #[cfg(creusot)]
        proof_assert!(weights(text@, from@, from@ + produced.len()) == n@ + lead_weight(byte));
        if byte < 0x80 || byte >= 0xc0 {
            n += 1;
        }
        if byte >= 0xf0 {
            n += 1;
        }
    }
    n
}
