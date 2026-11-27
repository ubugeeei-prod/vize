//! Creusot proves the cursor window of retained recovery records on its
//! shipped source (`window.rs`). This file is the library root of
//! `tests/formal/creusot/recovery-positions` and is not part of
//! `vize_resident_graph`'s module tree: the module below is the one the
//! product compiles, found by ordinary module discovery.
//!
//! `matching_positions` and its search `skip_below` carry their contracts in
//! the shipped source, and `skip_below`'s loop invariant discharges the
//! `requires` that creusot-std gives `<[T]>::get_unchecked` (`ix.in_bounds`)
//! at both unchecked reads. Neither function's safety depends on its input:
//! the bounds hold for unsorted cursors too, and only the window's meaning
//! needs sorted ones. The law below restates the window from its declarative
//! meaning, as the bounded Kani harness it replaces did, but for cursor lists
//! of every length, every `u32` endpoint and reversed records as well.
//! See docs/davinci/decisions/2026-11-24-unsafe-model-checking.md.

mod window;

use core::ops::Range;
use creusot_std::prelude::*;
use window::{matching_positions, sorted};

/// For sorted cursors, the window is in bounds, starts after exactly the
/// cursors before the record, and holds exactly the cursors the record
/// covers: a nonempty record its half-open interval, an empty record the
/// cursors at its position. A reversed record covers none.
#[requires(sorted(points@))]
#[ensures(result.start@ <= result.end@ && result.end@ <= points@.len())]
#[ensures(forall<i: Int> 0 <= i && i < points@.len()
    ==> (i < result.start@) == (points@[i]@ < start@))]
#[ensures(forall<i: Int> 0 <= i && i < points@.len()
    ==> (result.start@ <= i && i < result.end@)
        == ((start@ <= points@[i]@ && points@[i]@ < end@) || (start == end && points@[i] == start)))]
pub fn window_matches_each_requested_cursor(points: &[u32], start: u32, end: u32) -> Range<usize> {
    matching_positions(points, start, end)
}
