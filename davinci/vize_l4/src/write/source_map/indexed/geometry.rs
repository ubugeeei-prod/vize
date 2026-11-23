//! Coordinate arithmetic shared with the machine-word proof harness.
pub(super) const STRIDE: usize = 128;

pub(super) const fn checkpoint(offset: usize) -> usize {
    offset - offset % STRIDE
}

// At a scalar boundary, UTF-8 has one to four bytes and UTF-16 has one or
// two units. The cumulative byte excess is therefore never negative.
pub(super) const fn prefix_units(start: usize, excess: usize, tail_units: usize) -> usize {
    start - excess + tail_units
}
