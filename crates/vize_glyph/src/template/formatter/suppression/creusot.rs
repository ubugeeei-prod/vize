//! Creusot proves the template formatter's line-break decisions between two
//! chunks on their shipped source (`gap.rs`). This file is the library root
//! of `tests/formal/creusot/glyph-line-breaks` and is not part of
//! `vize_glyph`'s module tree: the module below is the one the product
//! compiles, found by ordinary module discovery.
//!
//! `LineJoiner::open` asks `is_single_line_break_gap` first and falls back to
//! `whitespace_gap_line_breaks`. Both carry their contracts in the shipped
//! source, stated over the input stream's newline normalization (HTML
//! §13.2.3.5: a CR LF pair and a lone CR each become one LF). The laws below
//! are stated over those contracts and hold for gaps of every length.
//! See docs/davinci/decisions/2026-11-17-creusot.md.

mod gap;

use creusot_std::prelude::*;
use gap::{blank, breaks, is_single_line_break_gap, whitespace_gap_line_breaks};

/// The fast path agrees with the general count: a gap it accepts is blank
/// and holds exactly one line break, so `open` takes the same decision for
/// it either way.
#[ensures(result ==> whitespace_gap_line_breaks_spec(gap@) == Some(1))]
pub fn a_single_line_break_gap_counts_one_break(gap: &[u8]) -> bool {
    is_single_line_break_gap(gap)
}

/// Every gap of whitespace gets a count, and only such gaps do: the count
/// is the normalized line breaks, saturated at `u8::MAX`.
#[ensures(match result {
    None => !blank(gap@, gap@.len()),
    Some(count) => blank(gap@, gap@.len()) && count@ == saturated(breaks(gap@, gap@.len())),
})]
pub fn a_blank_gap_counts_its_normalized_line_breaks(gap: &[u8]) -> Option<u8> {
    whitespace_gap_line_breaks(gap)
}

/// More whitespace never removes a line break: appending a byte to a gap
/// never lowers its count.
#[logic]
#[requires(0 <= len && len < gap.len())]
#[ensures(breaks(gap, len) <= breaks(gap, len + 1) && breaks(gap, len + 1) <= breaks(gap, len) + 1)]
pub fn line_breaks_grow_by_at_most_one_per_byte(gap: Seq<u8>, len: Int) {}

#[logic(open(crate))]
pub(crate) fn saturated(count: Int) -> Int {
    pearlite! { if count < 255 { count } else { 255 } }
}

/// What `whitespace_gap_line_breaks` returns for `gap`, by its contract.
#[logic(open(crate))]
pub(crate) fn whitespace_gap_line_breaks_spec(gap: Seq<u8>) -> Option<Int> {
    pearlite! {
        if blank(gap, gap.len()) { Some(saturated(breaks(gap, gap.len()))) } else { None }
    }
}
