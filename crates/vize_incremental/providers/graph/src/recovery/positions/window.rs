use core::ops::Range;

#[inline]
pub(super) fn matching_position(cursor: u32, start: u32, end: u32) -> Range<usize> {
    let before = usize::from(cursor < start);
    let inside = (start == end && cursor == start) || (start <= cursor && cursor < end);
    before..before + usize::from(inside)
}

/// Up to four requested positions fit in one 16-byte group.
#[inline]
pub(super) fn matching_short(positions: &[u32], start: u32, end: u32) -> Range<usize> {
    let last = end - u32::from(start < end);
    let mut lower = 0;
    let mut upper = 0;
    for &point in positions {
        lower += usize::from(point < start);
        upper += usize::from(point <= last);
    }
    lower..upper
}

#[inline]
pub(super) fn matching_positions(positions: &[u32], start: u32, end: u32) -> Range<usize> {
    let Some((&low, &high)) = positions.first().zip(positions.last()) else {
        return 0..0;
    };
    let length = positions.len();
    if high < start {
        return length..length;
    }
    // Insertion points include their only byte position. A nonempty span's
    // right endpoint is excluded, including when the left endpoint is zero.
    let last = end - u32::from(start < end);
    if last < low {
        return 0..0;
    }
    let first = if low < start {
        positions.partition_point(|&p| p < start)
    } else {
        0
    };
    if high <= last {
        return first..length;
    }
    let rest = &positions[first..];
    // Most source records cover only a few cursors. Search beyond the
    // inline prefix only when its last position also belongs to this record.
    let count = match rest {
        [point, ..] if *point > last => 0,
        [_, next, ..] if *next > last => 1,
        [_, _, _, fourth, tail @ ..] if *fourth <= last => {
            4 + tail.partition_point(|&point| point <= last)
        }
        [_, _, third, ..] => 2 + usize::from(*third <= last),
        _ => rest.len(),
    };
    first..first + count
}
