//! The cursor window of one recovery record: which of the caller's sorted
//! cursors the record covers. Creusot proves the contracts below on this
//! source, for every length and every `u32` cursor (see `creusot.rs` beside
//! this file). The proof is what makes the unchecked reads in `skip_below`
//! sound, so neither read has a precondition its callers must meet.
use core::{hint::select_unpredictable, ops::Range};
#[cfg(creusot)]
use creusot_std::prelude::*;

/// The indices of the sorted `positions` that a record at `start..end`
/// covers. The window starts at the first cursor at or after `start`. A
/// nonempty record covers its half-open interval, and an empty record covers
/// the cursors at its own position, EOF included. Duplicate cursors stay in
/// the window.
#[cfg_attr(creusot, ensures(result.start@ <= result.end@ && result.end@ <= positions@.len()))]
#[cfg_attr(creusot, ensures(sorted(positions@) ==> forall<i: Int> 0 <= i && i < positions@.len()
    ==> (i < result.start@) == (positions@[i]@ < start@)))]
#[cfg_attr(creusot, ensures(sorted(positions@) ==> forall<i: Int> 0 <= i && i < positions@.len()
    ==> (result.start@ <= i && i < result.end@) == covers(start@, end@, positions@[i]@)))]
#[inline]
pub(super) fn matching_positions(positions: &[u32], start: u32, end: u32) -> Range<usize> {
    let first = skip_below(positions, 0, start, false);
    let last = if start == end {
        skip_below(positions, first, end, true)
    } else {
        skip_below(positions, first, end, false)
    };
    first..last
}

/// The first index at or after `from` whose cursor is not below `bound`
/// (below or at it when `inclusive`), or the length when there is none.
/// The search is the one `partition_point` runs: its iteration count
/// depends only on the length, and each step selects without a branch.
#[cfg_attr(creusot, requires(from@ <= positions@.len()))]
#[cfg_attr(creusot, ensures(from@ <= result@ && result@ <= positions@.len()))]
#[cfg_attr(creusot, ensures(sorted(positions@) ==> forall<i: Int> from@ <= i && i < positions@.len()
    ==> (i < result@) == below(positions@[i]@, bound@, inclusive)))]
#[inline]
fn skip_below(positions: &[u32], from: usize, bound: u32, inclusive: bool) -> usize {
    let mut base = from;
    let mut size = positions.len() - from;
    if size == 0 {
        return base;
    }
    #[cfg_attr(creusot, invariant(from@ <= base@ && 1 <= size@ && base@ + size@ <= positions@.len()))]
    #[cfg_attr(creusot, invariant(sorted(positions@) ==> forall<i: Int> from@ <= i && i < base@
        ==> below(positions@[i]@, bound@, inclusive)))]
    #[cfg_attr(creusot, invariant(sorted(positions@) ==> forall<i: Int> base@ + size@ <= i && i < positions@.len()
        ==> !below(positions@[i]@, bound@, inclusive)))]
    #[cfg_attr(creusot, variant(size@))]
    while size > 1 {
        let half = size / 2;
        let mid = base + half;
        // SAFETY: `mid < base + size <= positions.len()`, because `half < size`
        // and `base + size <= positions.len()` is the first loop invariant of
        // `skip_below`. Creusot proves that invariant and discharges
        // `get_unchecked`'s `requires` (`ix.in_bounds`) from it.
        let point = unsafe { *positions.get_unchecked(mid) };
        base = select_unpredictable(point < bound || (inclusive && point == bound), mid, base);
        size -= half;
    }
    // SAFETY: the loop leaves `size == 1`, so `base < base + size <=
    // positions.len()` by the same Creusot-proved invariant of `skip_below`.
    let point = unsafe { *positions.get_unchecked(base) };
    if point < bound || (inclusive && point == bound) {
        base + 1
    } else {
        base
    }
}

/// The cursors are in source order; duplicates are allowed.
#[cfg(creusot)]
#[logic(open(crate))]
pub(crate) fn sorted(points: Seq<u32>) -> bool {
    pearlite! {
        forall<i: Int, j: Int> 0 <= i && i <= j && j < points.len() ==> points[i]@ <= points[j]@
    }
}

/// A record at `start..end` covers `point`: a nonempty record its half-open
/// interval, an empty record its own position.
#[cfg(creusot)]
#[logic(open(crate))]
pub(crate) fn covers(start: Int, end: Int, point: Int) -> bool {
    pearlite! { (start <= point && point < end) || (start == end && point == start) }
}

/// `point` lies before `bound`, or at it when `inclusive`.
#[cfg(creusot)]
#[logic(open(crate))]
pub(crate) fn below(point: Int, bound: Int, inclusive: bool) -> bool {
    pearlite! { point < bound || (inclusive && point == bound) }
}
