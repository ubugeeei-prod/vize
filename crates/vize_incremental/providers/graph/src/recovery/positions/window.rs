use core::ops::Range;

pub(super) fn matching_positions(positions: &[u32], start: u32, end: u32) -> Range<usize> {
    let first = positions.partition_point(|&p| p < start);
    let rest = &positions[first..];
    let count = if start == end {
        rest.partition_point(|&p| p <= end)
    } else {
        rest.partition_point(|&p| p < end)
    };
    first..first + count
}
