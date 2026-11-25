//! UTF-8 well-formedness as the Unicode Standard states it (chapter 3, D92
//! and Table 3-7, "Well-Formed UTF-8 Byte Sequences"), and the laws the
//! unchecked `from_utf8` conversions rest on.
//!
//! A byte sequence is well-formed when it splits, from the front, into
//! well-formed scalar encodings. Table 3-7 lists them by first byte:
//!
//! | First      | Second     | Third      | Fourth     |
//! | ---------- | ---------- | ---------- | ---------- |
//! | `00..=7F`  |            |            |            |
//! | `C2..=DF`  | `80..=BF`  |            |            |
//! | `E0`       | `A0..=BF`  | `80..=BF`  |            |
//! | `E1..=EC`  | `80..=BF`  | `80..=BF`  |            |
//! | `ED`       | `80..=9F`  | `80..=BF`  |            |
//! | `EE..=EF`  | `80..=BF`  | `80..=BF`  |            |
//! | `F0`       | `90..=BF`  | `80..=BF`  | `80..=BF`  |
//! | `F1..=F3`  | `80..=BF`  | `80..=BF`  | `80..=BF`  |
//! | `F4`       | `80..=8F`  | `80..=BF`  | `80..=BF`  |
//!
//! `scalar` transcribes the table and `well_formed_range` the split. The
//! lemmas are the composition laws every unchecked builder uses: pieces
//! concatenate, an ASCII byte is a whole scalar, and an ASCII byte of a
//! well-formed sequence is a boundary, so a builder may cut the sequence
//! there. They hold for sequences of every length (Creusot, see `creusot.rs`
//! beside this file).
//!
//! `is_well_formed` is the executable reading of the definition. Creusot
//! proves it decides `well_formed`, and Kani checks that it agrees with
//! `core::str::from_utf8` on every sequence of up to four bytes (`kani.rs`),
//! which ties the transcription to the standard library's decoder.
//!
//! The product never compiles this module. It is the kernel of the
//! standalone Kani root `kani.rs` and of the Creusot package
//! `tests/formal/creusot/utf8` (`creusot.rs`), both beside it.

#[cfg(creusot)]
use creusot_std::prelude::*;

/// Whether `s[k]` exists below the bound `end` and lies in `low..=high`.
#[cfg(creusot)]
#[logic(open)]
pub fn byte_in(s: Seq<u8>, k: Int, end: Int, low: Int, high: Int) -> bool {
    pearlite! { 0 <= k && k < end && k < s.len() && low <= s[k]@ && s[k]@ <= high }
}

/// The length of the well-formed scalar encoding that starts at `s[at]` and
/// ends at or before `end`, or 0 when Table 3-7 has no such row.
#[cfg(creusot)]
#[logic(open)]
pub fn scalar(s: Seq<u8>, at: Int, end: Int) -> Int {
    pearlite! {
        if !(0 <= at && at < end && at < s.len()) {
            0
        } else if s[at]@ <= 0x7F {
            1
        } else if 0xC2 <= s[at]@ && s[at]@ <= 0xDF {
            if byte_in(s, at + 1, end, 0x80, 0xBF) { 2 } else { 0 }
        } else if s[at]@ == 0xE0 {
            if byte_in(s, at + 1, end, 0xA0, 0xBF) && byte_in(s, at + 2, end, 0x80, 0xBF) { 3 } else { 0 }
        } else if (0xE1 <= s[at]@ && s[at]@ <= 0xEC) || s[at]@ == 0xEE || s[at]@ == 0xEF {
            if byte_in(s, at + 1, end, 0x80, 0xBF) && byte_in(s, at + 2, end, 0x80, 0xBF) { 3 } else { 0 }
        } else if s[at]@ == 0xED {
            if byte_in(s, at + 1, end, 0x80, 0x9F) && byte_in(s, at + 2, end, 0x80, 0xBF) { 3 } else { 0 }
        } else if s[at]@ == 0xF0 {
            if byte_in(s, at + 1, end, 0x90, 0xBF)
                && byte_in(s, at + 2, end, 0x80, 0xBF)
                && byte_in(s, at + 3, end, 0x80, 0xBF) { 4 } else { 0 }
        } else if 0xF1 <= s[at]@ && s[at]@ <= 0xF3 {
            if byte_in(s, at + 1, end, 0x80, 0xBF)
                && byte_in(s, at + 2, end, 0x80, 0xBF)
                && byte_in(s, at + 3, end, 0x80, 0xBF) { 4 } else { 0 }
        } else if s[at]@ == 0xF4 {
            if byte_in(s, at + 1, end, 0x80, 0x8F)
                && byte_in(s, at + 2, end, 0x80, 0xBF)
                && byte_in(s, at + 3, end, 0x80, 0xBF) { 4 } else { 0 }
        } else {
            0
        }
    }
}

/// Whether `s[start..end]` splits, from the front, into well-formed scalar
/// encodings.
#[cfg(creusot)]
#[logic(open)]
#[variant(end - start)]
pub fn well_formed_range(s: Seq<u8>, start: Int, end: Int) -> bool {
    pearlite! {
        if end <= start {
            start == end
        } else {
            scalar(s, start, end) > 0 && well_formed_range(s, start + scalar(s, start, end), end)
        }
    }
}

/// Whether `s` is well-formed UTF-8 (Unicode D92).
#[cfg(creusot)]
#[logic(open)]
pub fn well_formed(s: Seq<u8>) -> bool {
    pearlite! { well_formed_range(s, 0, s.len()) }
}

/// `bytes[at + offset]`, if it exists.
#[cfg_attr(creusot, requires(at@ <= bytes@.len()))]
#[cfg_attr(creusot, ensures(at@ + offset@ < bytes@.len() ==> result == Some(bytes[at@ + offset@])))]
#[cfg_attr(creusot, ensures(bytes@.len() <= at@ + offset@ ==> result == None))]
fn byte_at(bytes: &[u8], at: usize, offset: usize) -> Option<u8> {
    if offset < bytes.len() - at {
        Some(bytes[at + offset])
    } else {
        None
    }
}

/// Whether `byte` is in `low..=high`.
#[cfg_attr(creusot, ensures(result == (byte != None && low@ <= byte.unwrap_logic()@ && byte.unwrap_logic()@ <= high@)))]
fn within(byte: Option<u8>, low: u8, high: u8) -> bool {
    match byte {
        Some(byte) => low <= byte && byte <= high,
        None => false,
    }
}

/// The length of the well-formed scalar encoding at `bytes[at]`, or 0.
#[cfg_attr(creusot, requires(at@ < bytes@.len()))]
#[cfg_attr(creusot, ensures(result@ == scalar(bytes@, at@, bytes@.len())))]
pub fn scalar_len(bytes: &[u8], at: usize) -> usize {
    let first = bytes[at];
    let second = byte_at(bytes, at, 1);
    let third = byte_at(bytes, at, 2);
    let fourth = byte_at(bytes, at, 3);
    let (ok, len) = match first {
        0x00..=0x7F => (true, 1),
        0xC2..=0xDF => (within(second, 0x80, 0xBF), 2),
        0xE0 => (within(second, 0xA0, 0xBF) && within(third, 0x80, 0xBF), 3),
        0xE1..=0xEC | 0xEE..=0xEF => (within(second, 0x80, 0xBF) && within(third, 0x80, 0xBF), 3),
        0xED => (within(second, 0x80, 0x9F) && within(third, 0x80, 0xBF), 3),
        0xF0 => (
            within(second, 0x90, 0xBF) && within(third, 0x80, 0xBF) && within(fourth, 0x80, 0xBF),
            4,
        ),
        0xF1..=0xF3 => (
            within(second, 0x80, 0xBF) && within(third, 0x80, 0xBF) && within(fourth, 0x80, 0xBF),
            4,
        ),
        0xF4 => (
            within(second, 0x80, 0x8F) && within(third, 0x80, 0xBF) && within(fourth, 0x80, 0xBF),
            4,
        ),
        _ => (false, 0),
    };
    if ok { len } else { 0 }
}

/// Whether `bytes` is well-formed UTF-8, read scalar by scalar.
#[cfg_attr(creusot, ensures(result == well_formed(bytes@)))]
pub fn is_well_formed(bytes: &[u8]) -> bool {
    let mut at = 0;
    #[cfg_attr(creusot, invariant(at@ <= bytes@.len()))]
    #[cfg_attr(creusot, invariant(well_formed(bytes@) == well_formed_range(bytes@, at@, bytes@.len())))]
    #[cfg_attr(creusot, variant(bytes@.len() - at@))]
    while at < bytes.len() {
        let len = scalar_len(bytes, at);
        if len == 0 {
            return false;
        }
        at += len;
    }
    true
}
