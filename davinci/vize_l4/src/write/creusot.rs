//! Creusot proves the source-map coordinate kernels of the L4 emitter on
//! their shipped source: the JavaScript line-terminator step
//! (`line_endings.rs`), and the line table, offset lookup and segment
//! resolution behind `SourceMapBuilder::finish` (`source_positions.rs`). This
//! file is the library root of `tests/formal/creusot/source-map` and is not
//! part of `vize_l4`'s module tree, as `kani.rs` beside it was not: the
//! modules below are the ones `write.rs` compiles, found by ordinary module
//! discovery.
//!
//! Line starts are stated here from ECMAScript's LineTerminatorSequence
//! (§12.3): LF, CR not followed by LF, LS, PS, and CR LF once. They refine
//! `starts` in `tests/formal/source-map/LineBreaks.lean`, extended with LS and
//! PS. The UTF-8 and UTF-16 facts come from the Unicode Standard
//! (`tests/formal/creusot/unicode`). The proofs hold for texts, tables and
//! segment lists of every length. See
//! docs/davinci/decisions/2026-11-17-creusot.md.

extern crate alloc;

mod line_endings;
mod source_positions;

use creusot_std::prelude::*;
use davinci_unicode::{
    boundaries_are_the_lead_bytes, boundaries_compose, boundary, is_trail, the_end_is_a_boundary,
    utf8_from,
};

/// Table 3-6: the three-byte UTF-8 form of `scalar` (U+0800..U+FFFF),
/// `1110xxxx 10yyyyyy 10zzzzzz`, begins at `at`.
#[logic(open)]
pub fn three_byte_form_at(s: Seq<u8>, at: Int, scalar: Int) -> bool {
    pearlite! {
        0 <= at && at + 2 < s.len()
            && s[at]@ == 224 + scalar / 4096
            && s[at + 1]@ == 128 + (scalar / 64) % 64
            && s[at + 2]@ == 128 + scalar % 64
    }
}

/// LINE SEPARATOR (U+2028) or PARAGRAPH SEPARATOR (U+2029) begins at `at`.
#[logic(open)]
pub fn separator_at(s: Seq<u8>, at: Int) -> bool {
    pearlite! { three_byte_form_at(s, at, 8232) || three_byte_form_at(s, at, 8233) }
}

/// ECMAScript LineTerminatorSequence: the width of the one that begins at
/// `at`, or 0. CR LF is one sequence of two bytes.
#[logic(open)]
pub fn terminator_width(s: Seq<u8>, at: Int) -> Int {
    pearlite! {
        if at < 0 || at >= s.len() {
            0
        } else if s[at]@ == 13 {
            if at + 1 < s.len() && s[at + 1]@ == 10 { 2 } else { 1 }
        } else if s[at]@ == 10 {
            1
        } else if separator_at(s, at) {
            3
        } else {
            0
        }
    }
}

/// A line starts at `q` when a terminator sequence ends there: after an LF,
/// after a CR that no LF follows, or after LS or PS. Lean `starts` lists
/// these offsets for CR and LF.
#[logic(open)]
pub fn is_line_start(s: Seq<u8>, q: Int) -> bool {
    pearlite! {
        1 <= q && q <= s.len()
            && (s[q - 1]@ == 10
                || (s[q - 1]@ == 13 && !(q < s.len() && s[q]@ == 10))
                || separator_at(s, q - 3))
    }
}

/// The line starts in `(0, to]`: the line `to` is on.
#[logic(open)]
#[variant(to)]
pub fn line_breaks(s: Seq<u8>, to: Int) -> Int {
    pearlite! {
        if to <= 0 { 0 } else { line_breaks(s, to - 1) + if is_line_start(s, to) { 1 } else { 0 } }
    }
}

/// The greatest line start at or before `to`, or 0.
#[logic(open)]
#[variant(to)]
pub fn last_start(s: Seq<u8>, to: Int) -> Int {
    pearlite! {
        if to <= 0 { 0 } else if is_line_start(s, to) { to } else { last_start(s, to - 1) }
    }
}

/// `starts` is the line table of `s`: 0, then every line start in order.
/// Its `k`-th entry is the `k`-th line start.
#[logic(open)]
pub fn js_table(s: Seq<u8>, starts: Seq<usize>) -> bool {
    pearlite! {
        starts.len() == line_breaks(s, s.len()) + 1
            && starts[0]@ == 0
            && forall<k: Int> 1 <= k && k < starts.len()
                ==> 1 <= starts[k]@ && starts[k]@ <= s.len()
                    && is_line_start(s, starts[k]@)
                    && line_breaks(s, starts[k]@) == k
    }
}

/// The greatest scalar boundary at or before `at`, clamped to the text.
#[logic(open)]
#[variant(at)]
pub fn floor(s: Seq<u8>, at: Int) -> Int {
    pearlite! {
        if at <= 0 {
            0
        } else if at > s.len() {
            floor(s, s.len())
        } else if boundary(s, 0, at) {
            at
        } else {
            floor(s, at - 1)
        }
    }
}

/// The scan never stops inside LS or PS. It may stop between the CR and the
/// LF of a CR LF, where no line starts.
#[logic(open)]
pub fn not_inside(s: Seq<u8>, at: Int) -> bool {
    pearlite! { !separator_at(s, at - 1) && !separator_at(s, at - 2) }
}

/// Line starts never decrease in number along the text.
#[logic]
#[variant(to - from)]
#[ensures(from <= to ==> line_breaks(s, from) <= line_breaks(s, to))]
pub fn line_breaks_grow(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            line_breaks_grow(s, from, to - 1)
        }
    }
}

/// No more lines start than bytes precede.
#[logic]
#[variant(at)]
#[requires(0 <= at)]
#[ensures(0 <= line_breaks(s, at) && line_breaks(s, at) <= at)]
pub fn line_breaks_fit(s: Seq<u8>, at: Int) {
    pearlite! {
        if at > 0 {
            line_breaks_fit(s, at - 1)
        }
    }
}

/// Each entry of the table is preceded by exactly as many line starts as
/// its index.
#[logic]
#[requires(js_table(s, starts) && 0 <= k && k < starts.len())]
#[ensures(line_breaks(s, starts[k]@) == k)]
#[ensures(k == 0 ==> starts[k]@ == 0)]
#[ensures(k > 0 ==> is_line_start(s, starts[k]@))]
pub fn a_start_follows_its_breaks(s: Seq<u8>, starts: Seq<usize>, k: Int) {}

/// The table increases strictly, so binary search applies to it.
#[logic]
#[requires(js_table(s, starts) && 0 <= i && i < j && j < starts.len())]
#[ensures(starts[i]@ < starts[j]@)]
pub fn the_line_starts_increase(s: Seq<u8>, starts: Seq<usize>, i: Int, j: Int) {
    pearlite! {
        a_start_follows_its_breaks(s, starts, i);
        a_start_follows_its_breaks(s, starts, j);
        line_breaks_grow(s, starts[j]@, starts[i]@)
    }
}

/// From a line start, or 0, the last start stays put until the next one.
#[logic]
#[variant(to - from)]
#[requires(0 <= from && from <= to && line_breaks(s, from) == line_breaks(s, to))]
#[requires(from == 0 || is_line_start(s, from))]
#[ensures(last_start(s, to) == from)]
pub fn the_last_start_stays(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            line_breaks_grow(s, from, to - 1);
            the_last_start_stays(s, from, to - 1)
        }
    }
}

/// The last start is 0 or a line start, at or before `to`, on the same
/// line.
#[logic]
#[variant(to)]
#[requires(0 <= to)]
#[ensures(0 <= last_start(s, to) && last_start(s, to) <= to)]
#[ensures(last_start(s, to) == 0 || is_line_start(s, last_start(s, to)))]
#[ensures(line_breaks(s, last_start(s, to)) == line_breaks(s, to))]
pub fn the_last_start_is_a_line_start(s: Seq<u8>, to: Int) {
    pearlite! {
        if to > 0 && !is_line_start(s, to) {
            the_last_start_is_a_line_start(s, to - 1)
        }
    }
}

/// An offset is on line `k` exactly when it lies between the start of line
/// `k` and the start of the next, and then line `k` starts at `starts[k]`.
#[logic]
#[requires(js_table(s, starts) && 0 <= k && k < starts.len() && 0 <= at && at <= s.len())]
#[ensures((line_breaks(s, at) == k)
    == (starts[k]@ <= at && (k + 1 < starts.len() ==> at < starts[k + 1]@)))]
#[ensures(line_breaks(s, at) == k ==> last_start(s, at) == starts[k]@)]
pub fn a_line_lies_between_its_starts(s: Seq<u8>, starts: Seq<usize>, k: Int, at: Int) {
    pearlite! {
        a_start_follows_its_breaks(s, starts, k);
        line_breaks_grow(s, starts[k]@, at);
        line_breaks_grow(s, at, s.len());
        if k > 0 {
            line_breaks_grow(s, at, starts[k]@ - 1)
        };
        if k + 1 < starts.len() {
            a_start_follows_its_breaks(s, starts, k + 1);
            line_breaks_grow(s, at, starts[k + 1]@ - 1);
            line_breaks_grow(s, starts[k + 1]@, at)
        };
        if line_breaks(s, at) == k && starts[k]@ <= at {
            the_last_start_stays(s, starts[k]@, at)
        }
    }
}

/// A line start of UTF-8 text is a scalar boundary: every terminator is a
/// sequence of whole scalars.
#[logic]
#[requires(utf8_from(s, 0) && (q == 0 || is_line_start(s, q)))]
#[ensures(boundary(s, 0, q))]
pub fn line_starts_are_boundaries(s: Seq<u8>, q: Int) {
    pearlite! {
        if q > 0 {
            if separator_at(s, q - 3) {
                boundaries_are_the_lead_bytes(s, 0, q - 3);
                boundaries_compose(s, 0, q - 3, q)
            } else {
                boundaries_are_the_lead_bytes(s, 0, q - 1);
                boundaries_compose(s, 0, q - 1, q)
            }
        }
    }
}

/// One step of the forward scan from `at`: past a terminator a line starts
/// and the line count grows by one; past any other byte neither moves. The
/// scan stays outside LS and PS.
#[logic]
#[requires(0 <= at && at < s.len() && not_inside(s, at))]
#[ensures({
    let width = terminator_width(s, at);
    let next = if width > 0 { at + width } else { at + 1 };
    next <= s.len() && not_inside(s, next)
        && (width > 0 ==> is_line_start(s, next)
            && line_breaks(s, next) == line_breaks(s, at) + 1 && last_start(s, next) == next)
        && (width == 0 ==> !is_line_start(s, at + 1)
            && line_breaks(s, at + 1) == line_breaks(s, at)
            && last_start(s, at + 1) == last_start(s, at))
})]
pub fn a_scan_step(s: Seq<u8>, at: Int) {}

/// Stopping between the CR and the LF of a CR LF keeps the scan on the CR's
/// line.
#[logic]
#[requires(0 <= at)]
#[ensures(at + 1 < s.len() && s[at]@ == 13 && s[at + 1]@ == 10 && not_inside(s, at)
    ==> not_inside(s, at + 1) && !is_line_start(s, at + 1)
        && line_breaks(s, at + 1) == line_breaks(s, at)
        && last_start(s, at + 1) == last_start(s, at))]
pub fn a_split_crlf_stays_on_its_line(s: Seq<u8>, at: Int) {}

/// `floor` is the greatest boundary at or before the clamped offset.
#[logic]
#[variant(at)]
#[requires(utf8_from(s, 0))]
#[ensures(0 <= floor(s, at) && floor(s, at) <= s.len())]
#[ensures(at >= 0 ==> floor(s, at) <= at)]
#[ensures(boundary(s, 0, floor(s, at)))]
#[ensures(forall<b: Int> floor(s, at) < b && b <= at && b <= s.len() ==> !boundary(s, 0, b))]
pub fn floor_is_the_greatest_boundary(s: Seq<u8>, at: Int) {
    pearlite! {
        if at > s.len() {
            the_end_is_a_boundary(s, 0);
            floor_is_the_greatest_boundary(s, s.len())
        } else if at > 0 && !boundary(s, 0, at) {
            floor_is_the_greatest_boundary(s, at - 1)
        }
    }
}

/// The floor never decreases as the offset grows.
#[logic]
#[requires(utf8_from(s, 0) && a <= b)]
#[ensures(floor(s, a) <= floor(s, b))]
pub fn the_floor_grows(s: Seq<u8>, a: Int, b: Int) {
    pearlite! {
        floor_is_the_greatest_boundary(s, a);
        floor_is_the_greatest_boundary(s, b)
    }
}

/// Inside the text, the floor moves back over trailing bytes only.
#[logic]
#[requires(utf8_from(s, 0) && 0 < at && at < s.len())]
#[ensures(floor(s, at) == if boundary(s, 0, at) { at } else { floor(s, at - 1) })]
#[ensures(boundary(s, 0, at) == !is_trail(s[at]))]
pub fn the_floor_skips_trailing_bytes(s: Seq<u8>, at: Int) {
    boundaries_are_the_lead_bytes(s, 0, at)
}

/// The end of UTF-8 text is its own floor.
#[logic]
#[requires(utf8_from(s, 0))]
#[ensures(floor(s, s.len()) == s.len())]
pub fn the_end_is_its_own_floor(s: Seq<u8>) {
    the_end_is_a_boundary(s, 0)
}
