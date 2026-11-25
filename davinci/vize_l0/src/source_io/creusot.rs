//! Creusot proves the composition laws of UTF-8 well-formedness (`utf8.rs`)
//! for sequences of every length, and that `is_well_formed` decides it.
//! This file is the library root of `tests/formal/creusot/utf8` and is not
//! part of `vize_l0`'s module tree.
//!
//! The unchecked conversions in the product (see
//! docs/davinci/decisions/2026-11-24-unsafe-model-checking.md) each claim
//! that their buffer is a concatenation of well-formed pieces and ASCII
//! bytes, or that they copied a source range whose ends sit at ASCII bytes.
//! The lemmas below are those claims' general step: concatenation keeps
//! well-formedness, an ASCII byte is a whole scalar, an ASCII byte of a
//! well-formed sequence is a boundary, and dropping a trailing ASCII byte
//! keeps well-formedness.

mod utf8;

use creusot_std::prelude::*;
pub use utf8::is_well_formed;
use utf8::{scalar, well_formed, well_formed_range};

/// A scalar found below one bound is the same scalar below any bound past
/// its end.
#[logic]
#[requires(scalar(s, at, end) > 0)]
#[requires(at + scalar(s, at, end) <= other)]
#[ensures(scalar(s, at, other) == scalar(s, at, end))]
pub fn a_scalar_does_not_depend_on_the_bound_past_it(s: Seq<u8>, at: Int, end: Int, other: Int) {}

/// Every byte after the first byte of a multi-byte scalar is a
/// continuation byte (`80..=BF`), never ASCII.
#[logic]
#[requires(scalar(s, at, end) > 0)]
#[requires(at < k && k < at + scalar(s, at, end))]
#[ensures(0x80 <= s[k]@ && s[k]@ <= 0xBF)]
pub fn the_tail_of_a_scalar_is_continuation_bytes(s: Seq<u8>, at: Int, end: Int, k: Int) {}

/// An ASCII byte is a whole scalar.
#[logic]
#[requires(0 <= at && at < end && at < s.len())]
#[requires(s[at]@ <= 0x7F)]
#[ensures(scalar(s, at, end) == 1)]
#[ensures(well_formed_range(s, at, at + 1))]
pub fn an_ascii_byte_is_a_whole_scalar(s: Seq<u8>, at: Int, end: Int) {
    pearlite! { a_scalar_does_not_depend_on_the_bound_past_it(s, at, end, at + 1) }
}

/// Two adjacent well-formed ranges form a well-formed range.
#[logic]
#[variant(middle - start)]
#[requires(0 <= start && start <= middle && middle <= end)]
#[requires(well_formed_range(s, start, middle))]
#[requires(well_formed_range(s, middle, end))]
#[ensures(well_formed_range(s, start, end))]
pub fn adjacent_well_formed_ranges_concatenate(s: Seq<u8>, start: Int, middle: Int, end: Int) {
    pearlite! {
        if start < middle {
            a_scalar_does_not_depend_on_the_bound_past_it(s, start, middle, end);
            adjacent_well_formed_ranges_concatenate(s, start + scalar(s, start, middle), middle, end)
        }
    }
}

/// The scalar at the front of a range reads only the bytes in the range.
#[logic]
#[requires(0 <= start && start < end && end <= s.len())]
#[requires(0 <= other && other + (end - start) <= t.len())]
#[requires(forall<k: Int> 0 <= k && k < end - start ==> s[start + k] == t[other + k])]
#[ensures(scalar(s, start, end) == scalar(t, other, other + (end - start)))]
pub fn a_scalar_is_read_from_its_bytes_alone(
    s: Seq<u8>,
    start: Int,
    end: Int,
    t: Seq<u8>,
    other: Int,
) {
    pearlite! {
        {
            proof_assert!(s[start] == t[other]);
            proof_assert!(start + 1 < end ==> s[start + 1] == t[other + 1]);
            proof_assert!(start + 2 < end ==> s[start + 2] == t[other + 2]);
            proof_assert!(start + 3 < end ==> s[start + 3] == t[other + 3]);
        }
    }
}

/// Well-formedness of a range reads only the bytes in it.
#[logic]
#[variant(end - start)]
#[requires(0 <= start && start <= end && end <= s.len())]
#[requires(0 <= other && other + (end - start) <= t.len())]
#[requires(forall<k: Int> 0 <= k && k < end - start ==> s[start + k] == t[other + k])]
#[ensures(well_formed_range(s, start, end) == well_formed_range(t, other, other + (end - start)))]
pub fn a_range_is_well_formed_by_its_bytes_alone(
    s: Seq<u8>,
    start: Int,
    end: Int,
    t: Seq<u8>,
    other: Int,
) {
    pearlite! {
        if start < end {
            {
                a_scalar_is_read_from_its_bytes_alone(s, start, end, t, other);
                if scalar(s, start, end) > 0 {
                    a_range_is_well_formed_by_its_bytes_alone(s, start + scalar(s, start, end), end, t, other + scalar(s, start, end))
                }
            }
        }
    }
}

/// Concatenating well-formed sequences yields a well-formed sequence.
#[logic]
#[requires(well_formed(a) && well_formed(b))]
#[ensures(well_formed(a.concat(b)))]
pub fn well_formed_sequences_concatenate(a: Seq<u8>, b: Seq<u8>) {
    pearlite! {
        {
            a_range_is_well_formed_by_its_bytes_alone(a, 0, a.len(), a.concat(b), 0);
            a_range_is_well_formed_by_its_bytes_alone(b, 0, b.len(), a.concat(b), a.len());
            adjacent_well_formed_ranges_concatenate(a.concat(b), 0, a.len(), a.len() + b.len())
        }
    }
}

/// Appending an ASCII byte to a well-formed sequence keeps it well-formed.
#[logic]
#[requires(well_formed(a))]
#[requires(byte@ <= 0x7F)]
#[ensures(well_formed(a.push_back(byte)))]
pub fn appending_an_ascii_byte_keeps_a_sequence_well_formed(a: Seq<u8>, byte: u8) {
    pearlite! {
        {
            an_ascii_byte_is_a_whole_scalar(Seq::singleton(byte), 0, 1);
            proof_assert!(a.push_back(byte) == a.concat(Seq::singleton(byte)));
            well_formed_sequences_concatenate(a, Seq::singleton(byte))
        }
    }
}

/// An ASCII byte inside a well-formed range splits it into two well-formed
/// ranges: a builder may cut a well-formed sequence at any ASCII byte.
#[logic]
#[variant(end - start)]
#[requires(0 <= start && start <= cut && cut < end && end <= s.len())]
#[requires(well_formed_range(s, start, end))]
#[requires(s[cut]@ <= 0x7F)]
#[ensures(well_formed_range(s, start, cut))]
#[ensures(well_formed_range(s, cut, end))]
pub fn an_ascii_byte_splits_a_well_formed_range(s: Seq<u8>, start: Int, cut: Int, end: Int) {
    pearlite! {
        if start < cut {
            {
                let len = scalar(s, start, end);
                if start + len > cut {
                    the_tail_of_a_scalar_is_continuation_bytes(s, start, end, cut)
                } else {
                    {
                        an_ascii_byte_splits_a_well_formed_range(s, start + len, cut, end);
                        a_scalar_does_not_depend_on_the_bound_past_it(s, start, end, cut)
                    }
                }
            }
        }
    }
}

/// Dropping a trailing ASCII byte keeps a sequence well-formed, the step of
/// every trailing-whitespace trim.
#[logic]
#[requires(well_formed(s) && s.len() > 0)]
#[requires(s[s.len() - 1]@ <= 0x7F)]
#[ensures(well_formed(s.subsequence(0, s.len() - 1)))]
pub fn dropping_a_trailing_ascii_byte_keeps_a_sequence_well_formed(s: Seq<u8>) {
    pearlite! {
        {
            an_ascii_byte_splits_a_well_formed_range(s, 0, s.len() - 1, s.len());
            a_range_is_well_formed_by_its_bytes_alone(s, 0, s.len() - 1, s.subsequence(0, s.len() - 1), 0)
        }
    }
}

/// A cut of a well-formed sequence at its ends, at an ASCII byte, or just
/// past one.
#[logic(open)]
pub fn ascii_boundary(s: Seq<u8>, at: Int) -> bool {
    pearlite! {
        at == 0 || at == s.len()
            || (0 < at && at < s.len() && s[at]@ <= 0x7F)
            || (0 < at && at <= s.len() && s[at - 1]@ <= 0x7F)
    }
}

/// A range of a well-formed sequence whose ends are ASCII boundaries is
/// well-formed: the copy step of every scanner that cuts its input at ASCII
/// tokens.
#[logic]
#[requires(well_formed(s))]
#[requires(0 <= from && from <= to && to <= s.len())]
#[requires(ascii_boundary(s, from) && ascii_boundary(s, to))]
#[ensures(well_formed_range(s, from, to))]
pub fn a_range_between_ascii_boundaries_is_well_formed(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        {
            a_suffix_from_an_ascii_boundary_is_well_formed(s, from);
            if from < to {
                if to < s.len() && s[to]@ <= 0x7F {
                    an_ascii_byte_splits_a_well_formed_range(s, from, to, s.len())
                } else if to < s.len() {
                    {
                        an_ascii_byte_splits_a_well_formed_range(s, from, to - 1, s.len());
                        an_ascii_byte_is_a_whole_scalar(s, to - 1, s.len());
                        adjacent_well_formed_ranges_concatenate(s, from, to - 1, to)
                    }
                }
            }
        }
    }
}

/// The suffix of a well-formed sequence from an ASCII boundary is
/// well-formed.
#[logic]
#[requires(well_formed(s))]
#[requires(0 <= at && at <= s.len() && ascii_boundary(s, at))]
#[ensures(well_formed_range(s, at, s.len()))]
pub fn a_suffix_from_an_ascii_boundary_is_well_formed(s: Seq<u8>, at: Int) {
    pearlite! {
        if 0 < at && at < s.len() {
            if s[at]@ <= 0x7F {
                an_ascii_byte_splits_a_well_formed_range(s, 0, at, s.len())
            } else {
                {
                    an_ascii_byte_splits_a_well_formed_range(s, 0, at - 1, s.len());
                    an_ascii_byte_is_a_whole_scalar(s, at - 1, s.len())
                }
            }
        }
    }
}
