//! LSP positions against a table of line starts: 0, then the offset after
//! every line feed, as `LineIndex::new` builds it. A line ends at its LF, so
//! a CR before the LF is an ordinary character of the line. Creusot proves
//! both directions for every UTF-8 text and every such table (`creusot.rs`
//! beside this file states the table and the lemmas).

#[cfg(creusot)]
use crate::{
    a_line_lies_between_its_starts, every_offset_is_on_a_line, lf_table, line_feeds, line_start,
    the_line_starts_are_boundaries, the_line_starts_increase, the_line_table_is_sorted,
};
#[cfg(creusot)]
use creusot_std::prelude::*;
#[cfg(creusot)]
use davinci_unicode::{
    a_later_boundary_follows, boundaries_compose, boundary, the_end_is_a_boundary, units, utf8_from,
};

use super::code_units::{offset_of_units, units_between};

/// The 0-indexed line of `offset` and its column in UTF-16 code units from
/// the line start. An offset past the end is clamped, and one inside a
/// scalar counts that scalar, so the lookup never panics.
#[cfg_attr(creusot, requires(utf8_from(bytes@, 0) && lf_table(bytes@, starts@)))]
#[cfg_attr(creusot, ensures({
    let at = if offset@ <= bytes@.len() { offset@ } else { bytes@.len() };
    result.0@ == line_feeds(bytes@, at) && result.1@ == units(bytes@, line_start(bytes@, at), at)
}))]
#[inline]
pub(super) fn line_col(bytes: &[u8], starts: &[usize], offset: usize) -> (usize, usize) {
    let offset = offset.min(bytes.len());
    #[cfg(creusot)]
    proof_assert!(the_line_table_is_sorted(bytes@, starts@);
        forall<i: Int, j: Int> 0 <= i && i < j && j < starts@.len() ==> starts@[i]@ < starts@[j]@);
    #[cfg(creusot)]
    proof_assert!(starts.deep_model().sorted());
    // The greatest line start at or before `offset`. The table begins with
    // 0, so a predecessor always exists.
    let line = match starts.binary_search(&offset) {
        Ok(line) => line,
        Err(next) => next.saturating_sub(1),
    };
    let start = match starts.get(line) {
        Some(&start) => start,
        None => 0,
    };
    #[cfg(creusot)]
    proof_assert!(line@ < starts@.len() && start == starts@[line@] && start@ <= offset@);
    #[cfg(creusot)]
    proof_assert!(a_line_lies_between_its_starts(bytes@, starts@, line@, offset@);
        line_feeds(bytes@, offset@) == line@ && line_start(bytes@, offset@) == start@);
    #[cfg(creusot)]
    proof_assert!(the_line_starts_are_boundaries(bytes@, starts@, line@); utf8_from(bytes@, start@));
    let column = units_between(bytes, start, offset);
    #[cfg(creusot)]
    proof_assert!(column@ == units(bytes@, line_start(bytes@, offset@), offset@));
    (line, column)
}

/// The offset of the scalar boundary on `line` that `column` UTF-16 code
/// units of that line precede, or `None` when the line does not exist or no
/// boundary on it has that column. The line's own LF is its last position.
#[cfg_attr(creusot, requires(utf8_from(bytes@, 0) && lf_table(bytes@, starts@)))]
#[cfg_attr(creusot, ensures(match result {
    Some(at) => at@ <= bytes@.len() && boundary(bytes@, 0, at@)
        && line_feeds(bytes@, at@) == line@
        && units(bytes@, line_start(bytes@, at@), at@) == column@,
    None => forall<at: Int> 0 <= at && at <= bytes@.len() && boundary(bytes@, 0, at)
        && line_feeds(bytes@, at) == line@
        ==> units(bytes@, line_start(bytes@, at), at) != column@,
}))]
#[inline]
pub(super) fn offset_at(
    bytes: &[u8],
    starts: &[usize],
    line: usize,
    column: usize,
) -> Option<usize> {
    #[cfg(creusot)]
    proof_assert!(forall<at: Int> 0 <= at && at <= bytes@.len()
        ==> { every_offset_is_on_a_line(bytes@, starts@, at); line_feeds(bytes@, at) < starts@.len() });
    let start = *starts.get(line)?;
    let end = match starts.get(line + 1) {
        Some(next) => next.saturating_sub(1),
        None => bytes.len(),
    };
    #[cfg(creusot)]
    proof_assert!(the_line_starts_are_boundaries(bytes@, starts@, line@); true);
    #[cfg(creusot)]
    proof_assert!({
        if line@ + 1 < starts@.len() {
            the_line_starts_increase(bytes@, starts@, line@, line@ + 1);
            the_line_starts_are_boundaries(bytes@, starts@, line@ + 1);
            a_later_boundary_follows(bytes@, 0, start@, end@)
        } else {
            the_end_is_a_boundary(bytes@, start@)
        };
        start@ <= end@ && boundary(bytes@, start@, end@)
    });
    #[cfg(creusot)]
    proof_assert!(forall<at: Int> 0 <= at && at <= bytes@.len()
        ==> { a_line_lies_between_its_starts(bytes@, starts@, line@, at);
            (line_feeds(bytes@, at) == line@) == (start@ <= at && at <= end@)
            && (line_feeds(bytes@, at) == line@ ==> line_start(bytes@, at) == start@) });
    #[cfg(creusot)]
    proof_assert!(forall<at: Int> start@ <= at && at <= end@ && boundary(bytes@, 0, at)
        ==> { a_later_boundary_follows(bytes@, 0, start@, at); boundary(bytes@, start@, at) });
    let found = offset_of_units(bytes, start, end, column);
    #[cfg(creusot)]
    proof_assert!(match found {
        Some(at) => {
            a_line_lies_between_its_starts(bytes@, starts@, line@, at@);
            boundaries_compose(bytes@, 0, start@, at@);
            true
        }
        None => true,
    });
    found
}
