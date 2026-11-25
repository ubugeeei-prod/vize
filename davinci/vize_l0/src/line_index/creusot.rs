//! Creusot proves the LSP coordinate kernels of `LineIndex` on their shipped
//! source: UTF-16 columns counted on UTF-8 bytes (`code_units.rs`) and the
//! line-table lookups behind `LineIndex::line_col` and
//! `LineIndex::line_col_to_offset` (`line_table.rs`). This file is the library
//! root of `tests/formal/creusot/line-index` and is not part of `vize_l0`'s
//! module tree, as `kani.rs` beside it is not: the modules below are the ones
//! `line_index.rs` compiles, found by ordinary module discovery.
//!
//! The UTF-8 and UTF-16 facts come from the Unicode Standard
//! (`tests/formal/creusot/unicode`). The line table is stated here from its
//! definition: 0, then the offset after each line feed. The proofs hold for
//! texts and tables of every length, where Kani ran out of memory at three
//! bytes. See docs/davinci/decisions/2026-11-17-creusot.md.

mod code_units;
mod line_table;

use creusot_std::prelude::*;
use davinci_unicode::{
    a_later_boundary_follows, a_scalar_widens_the_units, boundaries_are_the_lead_bytes,
    boundaries_compose, boundary, forms_continue_after_a_boundary, units,
    units_append, utf8_from,
};

/// The line feeds (U+000A, a one-byte form) in `s[0..to)`.
#[logic(open)]
#[variant(to)]
pub fn line_feeds(s: Seq<u8>, to: Int) -> Int {
    pearlite! {
        if to <= 0 { 0 } else { line_feeds(s, to - 1) + if s[to - 1]@ == 10 { 1 } else { 0 } }
    }
}

/// The start of the line that holds `to`: the offset after the last line
/// feed in `s[0..to)`, or 0.
#[logic(open)]
#[variant(to)]
pub fn line_start(s: Seq<u8>, to: Int) -> Int {
    pearlite! {
        if to <= 0 { 0 } else if s[to - 1]@ == 10 { to } else { line_start(s, to - 1) }
    }
}

/// `starts` is the line table of `s`: 0, then the offset after each line
/// feed, in order. Its `k`-th entry follows the `k`-th line feed.
#[logic(open)]
pub fn lf_table(s: Seq<u8>, starts: Seq<usize>) -> bool {
    pearlite! {
        starts.len() == line_feeds(s, s.len()) + 1
            && starts[0]@ == 0
            && forall<k: Int> 1 <= k && k < starts.len()
                ==> 1 <= starts[k]@ && starts[k]@ <= s.len()
                    && s[starts[k]@ - 1]@ == 10
                    && line_feeds(s, starts[k]@ - 1) == k - 1
    }
}

/// Line feeds never decrease along the text.
#[logic]
#[variant(to - from)]
#[ensures(from <= to ==> line_feeds(s, from) <= line_feeds(s, to))]
pub fn line_feeds_grow(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            line_feeds_grow(s, from, to - 1)
        }
    }
}

/// Each entry of the table follows exactly as many line feeds as its index.
#[logic]
#[requires(lf_table(s, starts) && 0 <= k && k < starts.len())]
#[ensures(line_feeds(s, starts[k]@) == k)]
pub fn a_start_follows_its_line_feeds(s: Seq<u8>, starts: Seq<usize>, k: Int) {}

/// The table increases strictly, so binary search applies to it.
#[logic]
#[requires(lf_table(s, starts) && 0 <= i && i < j && j < starts.len())]
#[ensures(starts[i]@ < starts[j]@)]
pub fn the_line_starts_increase(s: Seq<u8>, starts: Seq<usize>, i: Int, j: Int) {
    pearlite! {
        a_start_follows_its_line_feeds(s, starts, i);
        a_start_follows_its_line_feeds(s, starts, j);
        line_feeds_grow(s, starts[j]@, starts[i]@)
    }
}

/// The whole table is sorted, for binary search.
#[logic]
#[requires(lf_table(s, starts))]
#[ensures(forall<i: Int, j: Int> 0 <= i && i < j && j < starts.len() ==> starts[i]@ < starts[j]@)]
pub fn the_line_table_is_sorted(s: Seq<u8>, starts: Seq<usize>) {
    pearlite! {
        let _ = the_line_starts_increase;
    }
}

/// The offset after a line feed in UTF-8 text is a scalar boundary.
#[logic]
#[requires(utf8_from(s, 0) && 0 <= at && at < s.len() && s[at]@ == 10)]
#[ensures(boundary(s, 0, at + 1) && utf8_from(s, at + 1))]
pub fn a_line_feed_ends_a_scalar(s: Seq<u8>, at: Int) {
    pearlite! {
        boundaries_are_the_lead_bytes(s, 0, at);
        boundaries_compose(s, 0, at, at + 1);
        forms_continue_after_a_boundary(s, 0, at + 1)
    }
}

/// Every line starts on a scalar boundary.
#[logic]
#[requires(utf8_from(s, 0) && lf_table(s, starts) && 0 <= k && k < starts.len())]
#[ensures(boundary(s, 0, starts[k]@) && utf8_from(s, starts[k]@))]
pub fn the_line_starts_are_boundaries(s: Seq<u8>, starts: Seq<usize>, k: Int) {
    pearlite! {
        if k > 0 {
            a_line_feed_ends_a_scalar(s, starts[k]@ - 1)
        }
    }
}

/// From the offset after a line feed, or 0, the line start stays put until
/// the next line feed.
#[logic]
#[variant(to - from)]
#[requires(0 <= from && from <= to && line_feeds(s, from) == line_feeds(s, to))]
#[requires(from == 0 || s[from - 1]@ == 10)]
#[ensures(line_start(s, to) == from)]
pub fn the_line_start_stays_until_a_line_feed(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            line_feeds_grow(s, from, to - 1);
            the_line_start_stays_until_a_line_feed(s, from, to - 1)
        }
    }
}

/// The line start is 0 or follows a line feed, and no line feed lies
/// between it and `to`.
#[logic]
#[variant(to)]
#[requires(0 <= to)]
#[ensures(0 <= line_start(s, to) && line_start(s, to) <= to)]
#[ensures(line_start(s, to) == 0 || s[line_start(s, to) - 1]@ == 10)]
#[ensures(line_feeds(s, line_start(s, to)) == line_feeds(s, to))]
pub fn the_line_start_is_after_the_last_line_feed(s: Seq<u8>, to: Int) {
    pearlite! {
        if to > 0 && s[to - 1]@ != 10 {
            the_line_start_is_after_the_last_line_feed(s, to - 1)
        }
    }
}

/// An offset is on line `k` exactly when it lies between the start of line
/// `k` and the start of the next, and then line `k` starts at `starts[k]`.
#[logic]
#[requires(lf_table(s, starts) && 0 <= k && k < starts.len() && 0 <= at && at <= s.len())]
#[ensures((line_feeds(s, at) == k)
    == (starts[k]@ <= at && (k + 1 < starts.len() ==> at < starts[k + 1]@)))]
#[ensures(line_feeds(s, at) == k ==> line_start(s, at) == starts[k]@)]
pub fn a_line_lies_between_its_starts(s: Seq<u8>, starts: Seq<usize>, k: Int, at: Int) {
    pearlite! {
        a_start_follows_its_line_feeds(s, starts, k);
        line_feeds_grow(s, starts[k]@, at);
        line_feeds_grow(s, at, s.len());
        if k > 0 {
            line_feeds_grow(s, at, starts[k]@ - 1)
        };
        if k + 1 < starts.len() {
            a_start_follows_its_line_feeds(s, starts, k + 1);
            line_feeds_grow(s, at, starts[k + 1]@ - 1);
            line_feeds_grow(s, starts[k + 1]@, at)
        };
        if line_feeds(s, at) == k && starts[k]@ <= at {
            the_line_start_stays_until_a_line_feed(s, starts[k]@, at)
        }
    }
}

/// Every offset of the text is on a line of the table.
#[logic]
#[requires(lf_table(s, starts) && 0 <= at && at <= s.len())]
#[ensures(0 <= line_feeds(s, at) && line_feeds(s, at) < starts.len())]
pub fn every_offset_is_on_a_line(s: Seq<u8>, starts: Seq<usize>, at: Int) {
    pearlite! {
        line_feeds_grow(s, at, s.len());
        line_feeds_grow(s, 0, at)
    }
}

/// Lean `column_exact`: the column of an offset is the units of the whole
/// prefix minus the units before its line start.
#[logic]
#[requires(utf8_from(s, 0) && 0 <= at && at <= s.len())]
#[ensures(units(s, line_start(s, at), at) == units(s, 0, at) - units(s, 0, line_start(s, at)))]
pub fn the_column_counts_the_units_since_the_line_start(s: Seq<u8>, at: Int) {
    pearlite! {
        the_line_start_is_after_the_last_line_feed(s, at);
        if line_start(s, at) > 0 {
            a_line_feed_ends_a_scalar(s, line_start(s, at) - 1)
        };
        units_append(s, 0, line_start(s, at), at)
    }
}

/// `line_col_to_offset` inverts `line_col` on scalar boundaries: two
/// boundaries with the same line and column are the same offset. With the
/// contracts of `line_col` and `offset_at`, the boundary `offset_at` returns
/// for the position of a boundary is that boundary, and every boundary it
/// returns has the position it was asked for.
#[logic]
#[requires(utf8_from(s, 0) && 0 <= a && a <= s.len() && 0 <= b && b <= s.len())]
#[requires(boundary(s, 0, a) && boundary(s, 0, b))]
#[requires(line_feeds(s, a) == line_feeds(s, b))]
#[requires(units(s, line_start(s, a), a) == units(s, line_start(s, b), b))]
#[ensures(a == b)]
pub fn line_col_is_one_to_one_on_boundaries(s: Seq<u8>, a: Int, b: Int) {
    pearlite! {
        the_line_start_is_after_the_last_line_feed(s, a);
        the_line_start_is_after_the_last_line_feed(s, b);
        if a <= b {
            the_line_start_stays_until_a_line_feed(s, line_start(s, a), b)
        } else {
            the_line_start_stays_until_a_line_feed(s, line_start(s, b), a)
        };
        if line_start(s, a) > 0 {
            a_line_feed_ends_a_scalar(s, line_start(s, a) - 1)
        };
        a_later_boundary_follows(s, 0, line_start(s, a), a);
        a_later_boundary_follows(s, 0, line_start(s, a), b);
        if a < b {
            a_scalar_widens_the_units(s, line_start(s, a), a, b)
        } else if b < a {
            a_scalar_widens_the_units(s, line_start(s, a), b, a)
        }
    }
}
