//! The UTF-8 and UTF-16 facts the source-coordinate kernels are proved
//! against, written from the Unicode Standard, §3.9 "Unicode Encoding Forms",
//! and from no kernel. A text is a sequence of bytes; walking it from a scalar
//! boundary reads one UTF-8 form at a time, and each form says how many
//! UTF-16 code units its scalar takes.
//!
//! `tests/formal/source-map/Coordinates.lean` states the same facts over
//! lists of scalar widths (`ScalarWidth`, `bytes`, `units`, `total`). The
//! lemmas below mirror its theorems on byte sequences and name the theorem
//! each one refines. The kernels that count units on bytes, a lead byte at a
//! time, are tied to the scalar walk by `lead_bytes_count_the_units`.
//!
//! This crate is a library of logic only: it has no program function, and the
//! product never depends on it. Kernel contracts name it under `cfg(creusot)`.

use creusot_std::prelude::*;

/// D92 and Table 3-7 (Well-Formed UTF-8 Byte Sequences): the length of the
/// UTF-8 form that begins with `lead`. No well-formed form begins with a
/// trailing byte (`80..=BF`), `C0`, `C1` or `F5..=FF`; those give 0. Lean
/// `bytes`.
#[logic(open)]
pub fn scalar_len(lead: u8) -> Int {
    pearlite! {
        if lead@ <= 127 {
            1
        } else if 194 <= lead@ && lead@ <= 223 {
            2
        } else if 224 <= lead@ && lead@ <= 239 {
            3
        } else if 240 <= lead@ && lead@ <= 244 {
            4
        } else {
            0
        }
    }
}

/// Table 3-7: every byte of a UTF-8 form after its first is in `80..=BF`.
#[logic(open)]
pub fn is_trail(byte: u8) -> bool {
    pearlite! { 128 <= byte@ && byte@ <= 191 }
}

/// D91 (UTF-16): a scalar value up to U+FFFF is one code unit, and one above
/// it is a surrogate pair. Table 3-6 encodes exactly the supplementary
/// scalars U+10000..U+10FFFF in four bytes, so the form's length decides.
/// Lean `units`.
#[logic(open)]
pub fn scalar_units(lead: u8) -> Int {
    pearlite! { if scalar_len(lead) == 4 { 2 } else { 1 } }
}

/// The bytes from `at` to the end are a sequence of UTF-8 forms: each begins
/// with a lead byte and continues with exactly the trailing bytes its length
/// asks for. This is the part of well-formedness (D92) the coordinate laws
/// need; the range limits on second bytes (Table 3-7) never matter here. The
/// bytes of every `&str` satisfy it from 0.
#[logic(open)]
#[variant(s.len() - at)]
pub fn utf8_from(s: Seq<u8>, at: Int) -> bool {
    pearlite! {
        if at >= s.len() {
            at == s.len()
        } else {
            0 <= at
                && 0 < scalar_len(s[at])
                && at + scalar_len(s[at]) <= s.len()
                && (forall<j: Int> at < j && j < at + scalar_len(s[at]) ==> is_trail(s[j]))
                && utf8_from(s, at + scalar_len(s[at]))
        }
    }
}

/// Walking whole UTF-8 forms from `from` lands on `to`: `to` is a scalar
/// boundary of the text read from `from`.
#[logic(open)]
#[variant(to - from)]
pub fn boundary(s: Seq<u8>, from: Int, to: Int) -> bool {
    pearlite! {
        if from >= to {
            from == to
        } else {
            0 < scalar_len(s[from]) && boundary(s, from + scalar_len(s[from]), to)
        }
    }
}

/// The UTF-16 code units of the scalars that begin in `[from, to)`, walking
/// from `from`. A `to` inside a scalar counts that scalar whole. Lean
/// `total units`.
#[logic(open)]
#[variant(s.len() - from)]
pub fn units(s: Seq<u8>, from: Int, to: Int) -> Int {
    pearlite! {
        if from < 0 || from >= to || from >= s.len() || scalar_len(s[from]) == 0 {
            0
        } else {
            scalar_units(s[from]) + units(s, from + scalar_len(s[from]), to)
        }
    }
}

/// What the counting kernels add for one byte: a scalar's units on its lead
/// byte and nothing on its trailing bytes.
#[logic(open)]
pub fn lead_weight(byte: u8) -> Int {
    pearlite! { if is_trail(byte) { 0 } else if byte@ >= 240 { 2 } else { 1 } }
}

/// The sum of `lead_weight` over `[from, to)`.
#[logic(open)]
#[variant(to - from)]
pub fn weights(s: Seq<u8>, from: Int, to: Int) -> Int {
    pearlite! {
        if to <= from { 0 } else { weights(s, from, to - 1) + lead_weight(s[to - 1]) }
    }
}

/// Weights add over adjacent ranges.
#[logic]
#[variant(to - mid)]
#[requires(from <= mid && mid <= to)]
#[ensures(weights(s, from, to) == weights(s, from, mid) + weights(s, mid, to))]
pub fn weights_split(s: Seq<u8>, from: Int, mid: Int, to: Int) {
    pearlite! {
        if mid < to {
            weights_split(s, from, mid, to - 1)
        }
    }
}

/// Weights are never negative.
#[logic]
#[variant(to - from)]
#[ensures(weights(s, from, to) >= 0)]
pub fn weights_are_nonnegative(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            weights_are_nonnegative(s, from, to - 1)
        }
    }
}

/// Weights never decrease as the range grows to the right.
#[logic]
#[variant(to - mid)]
#[requires(from <= mid && mid <= to)]
#[ensures(weights(s, from, mid) <= weights(s, from, to))]
pub fn weights_grow(s: Seq<u8>, from: Int, mid: Int, to: Int) {
    pearlite! {
        if mid < to {
            weights_grow(s, from, mid, to - 1)
        }
    }
}

/// Only the bytes in the range count.
#[logic]
#[variant(to - from)]
#[requires(0 <= from && from <= to && to <= s.len() && to <= t.len())]
#[requires(forall<i: Int> from <= i && i < to ==> s[i] == t[i])]
#[ensures(weights(s, from, to) == weights(t, from, to))]
pub fn weights_read_the_range(s: Seq<u8>, t: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            weights_read_the_range(s, t, from, to - 1)
        }
    }
}

/// The same range of `s` read as the start of `s.subsequence(base, ..)`.
#[logic]
#[variant(to - from)]
#[requires(0 <= base && 0 <= from && from <= to && base + to <= s.len())]
#[ensures(weights(s, base + from, base + to) == weights(s.subsequence(base, s.len()), from, to))]
pub fn weights_shift(s: Seq<u8>, base: Int, from: Int, to: Int) {
    pearlite! {
        if from < to {
            weights_shift(s, base, from, to - 1)
        }
    }
}

/// Trailing bytes weigh nothing.
#[logic]
#[variant(to - from)]
#[requires(forall<i: Int> from <= i && i < to ==> is_trail(s[i]))]
#[ensures(weights(s, from, to) == 0)]
pub fn trailing_bytes_weigh_nothing(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            trailing_bytes_weigh_nothing(s, from, to - 1)
        }
    }
}

/// An ASCII byte is its own scalar and one code unit.
#[logic]
#[variant(to - from)]
#[requires(forall<i: Int> from <= i && i < to ==> s[i]@ < 128)]
#[ensures(from <= to ==> weights(s, from, to) == to - from)]
pub fn ascii_bytes_weigh_one_each(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            ascii_bytes_weigh_one_each(s, from, to - 1)
        }
    }
}

/// Counting lead bytes counts the units of the scalars that begin in the
/// range: a lead byte carries its scalar's units and its trailing bytes carry
/// none. This is the bridge from the byte-counting kernels to the scalar
/// walk.
#[logic]
#[variant(s.len() - from)]
#[requires(utf8_from(s, from))]
#[requires(from <= to && to <= s.len())]
#[ensures(weights(s, from, to) == units(s, from, to))]
pub fn lead_bytes_count_the_units(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            let width = scalar_len(s[from]);
            weights_split(s, from, from + 1, to);
            if from + width < to {
                weights_split(s, from + 1, from + width, to);
                trailing_bytes_weigh_nothing(s, from + 1, from + width);
                lead_bytes_count_the_units(s, from + width, to)
            } else {
                trailing_bytes_weigh_nothing(s, from + 1, to)
            }
        }
    }
}

/// The text from a boundary is still a sequence of UTF-8 forms.
#[logic]
#[variant(to - from)]
#[requires(utf8_from(s, from) && boundary(s, from, to) && to <= s.len())]
#[ensures(utf8_from(s, to))]
pub fn forms_continue_after_a_boundary(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            forms_continue_after_a_boundary(s, from + scalar_len(s[from]), to)
        }
    }
}

/// Boundaries compose: a boundary of the text read from a boundary is one of
/// the text read from the start.
#[logic]
#[variant(mid - from)]
#[requires(boundary(s, from, mid) && boundary(s, mid, to))]
#[ensures(boundary(s, from, to))]
pub fn boundaries_compose(s: Seq<u8>, from: Int, mid: Int, to: Int) {
    pearlite! {
        if from < mid {
            boundaries_compose(s, from + scalar_len(s[from]), mid, to)
        }
    }
}

/// Inside the text, a scalar boundary is exactly a byte that is not a
/// trailing byte (Table 3-7: lead and trailing bytes are disjoint ranges).
#[logic]
#[variant(s.len() - from)]
#[requires(utf8_from(s, from) && from <= at && at < s.len())]
#[ensures(boundary(s, from, at) == !is_trail(s[at]))]
pub fn boundaries_are_the_lead_bytes(s: Seq<u8>, from: Int, at: Int) {
    pearlite! {
        if from + scalar_len(s[from]) <= at {
            boundaries_are_the_lead_bytes(s, from + scalar_len(s[from]), at)
        }
    }
}

/// The end of the text is a boundary.
#[logic]
#[variant(s.len() - from)]
#[requires(utf8_from(s, from))]
#[ensures(boundary(s, from, s.len()))]
pub fn the_end_is_a_boundary(s: Seq<u8>, from: Int) {
    pearlite! {
        if from < s.len() {
            the_end_is_a_boundary(s, from + scalar_len(s[from]))
        }
    }
}

/// Lean `appended`: the units of a text are the units up to a boundary plus
/// the units after it.
#[logic]
#[variant(mid - from)]
#[requires(utf8_from(s, from) && boundary(s, from, mid) && mid <= to && mid <= s.len())]
#[ensures(units(s, from, to) == units(s, from, mid) + units(s, mid, to))]
pub fn units_append(s: Seq<u8>, from: Int, mid: Int, to: Int) {
    pearlite! {
        if from < mid {
            units_append(s, from + scalar_len(s[from]), mid, to)
        }
    }
}

/// Units are never negative.
#[logic]
#[variant(s.len() - from)]
#[ensures(units(s, from, to) >= 0)]
pub fn units_are_nonnegative(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if !(from < 0 || from >= to || from >= s.len() || scalar_len(s[from]) == 0) {
            units_are_nonnegative(s, from + scalar_len(s[from]), to)
        }
    }
}

/// Lean `accounted`: no scalar takes more UTF-16 code units than UTF-8
/// bytes (the units plus a nonnegative excess are the byte count), so the
/// units of the scalars up to a boundary never exceed the bytes before it.
#[logic]
#[variant(to - from)]
#[requires(utf8_from(s, from) && boundary(s, from, to) && to <= s.len())]
#[ensures(units(s, from, to) <= to - from)]
pub fn units_fit_before_a_boundary(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            units_fit_before_a_boundary(s, from + scalar_len(s[from]), to)
        }
    }
}

/// Lean `accounted`, for any end: the units of the scalars that begin from
/// `from` never exceed the bytes left in the text.
#[logic]
#[variant(s.len() - from)]
#[requires(utf8_from(s, from))]
#[ensures(0 <= units(s, from, to) && units(s, from, to) <= s.len() - from)]
pub fn units_fit(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to && from < s.len() {
            units_fit(s, from + scalar_len(s[from]), to)
        }
    }
}

/// A lead byte at a boundary weighs exactly its scalar's units.
#[logic]
#[requires(utf8_from(s, from) && boundary(s, from, at) && at < s.len())]
#[ensures(0 < scalar_len(s[at]) && lead_weight(s[at]) == scalar_units(s[at]))]
pub fn a_lead_byte_weighs_its_units(s: Seq<u8>, from: Int, at: Int) {
    forms_continue_after_a_boundary(s, from, at)
}

/// Past the scalar at a boundary `at`, every later boundary has at least
/// that scalar's units more: units strictly grow along the boundaries.
#[logic]
#[requires(utf8_from(s, from) && boundary(s, from, at) && boundary(s, from, to))]
#[requires(from <= at && at < to && to <= s.len())]
#[ensures(units(s, from, to) >= units(s, from, at) + scalar_units(s[at]))]
pub fn a_scalar_widens_the_units(s: Seq<u8>, from: Int, at: Int, to: Int) {
    pearlite! {
        forms_continue_after_a_boundary(s, from, at);
        units_append(s, from, at, to);
        units_are_nonnegative(s, at + scalar_len(s[at]), to)
    }
}

/// ASCII bytes are one-byte forms of one code unit each: every offset among
/// them is a boundary and has as many units as bytes before it.
#[logic]
#[variant(to - from)]
#[requires(0 <= from && from <= to && to <= s.len())]
#[requires(forall<i: Int> from <= i && i < to ==> s[i]@ < 128)]
#[ensures(boundary(s, from, to) && units(s, from, to) == to - from)]
pub fn ascii_bytes_walk_one_unit_each(s: Seq<u8>, from: Int, to: Int) {
    pearlite! {
        if from < to {
            ascii_bytes_walk_one_unit_each(s, from + 1, to)
        }
    }
}

/// Two boundaries of the text read from `from` also bound the text between
/// them: the walk from the first reaches the second.
#[logic]
#[requires(utf8_from(s, from) && boundary(s, from, at) && boundary(s, from, to))]
#[requires(from <= at && at <= to && to <= s.len())]
#[ensures(boundary(s, at, to) && utf8_from(s, at))]
pub fn a_later_boundary_follows(s: Seq<u8>, from: Int, at: Int, to: Int) {
    pearlite! {
        forms_continue_after_a_boundary(s, from, at);
        if to < s.len() {
            boundaries_are_the_lead_bytes(s, from, to);
            boundaries_are_the_lead_bytes(s, at, to)
        } else {
            the_end_is_a_boundary(s, at)
        }
    }
}
