//! UTF-16 code units of UTF-8 text, counted on its bytes.
//!
//! A UTF-8 form is a lead byte followed by the trailing bytes `80..=BF` its
//! length asks for, and its scalar takes two UTF-16 code units exactly when
//! the form is four bytes long (Unicode §3.9, Tables 3-6 and 3-7). So the
//! units of the scalars that begin in a range are one per lead byte in it,
//! plus one more per four-byte lead (`F0..`): no scalar is decoded, and both
//! loops vectorize. Creusot proves each contract against the scalar-by-scalar
//! definitions in `tests/formal/creusot/unicode`, for UTF-8 text of every
//! length (`creusot.rs` beside this file). The `&str` entry points in
//! `utf16.rs` pass their bytes, which are always UTF-8.

#[cfg(creusot)]
use creusot_std::prelude::*;
#[cfg(creusot)]
use davinci_unicode::{
    a_lead_byte_weighs_its_units, a_scalar_widens_the_units, ascii_bytes_walk_one_unit_each,
    ascii_bytes_weigh_one_each, boundaries_are_the_lead_bytes, boundary, is_trail,
    lead_bytes_count_the_units, lead_weight, scalar_units, units, units_fit,
    units_fit_before_a_boundary, utf8_from, weights, weights_grow,
};

/// Whether every byte is ASCII. A running maximum vectorizes; std's
/// `is_ascii` has no Creusot specification.
#[cfg_attr(creusot, ensures(result == forall<i: Int> 0 <= i && i < bytes@.len() ==> bytes@[i]@ < 128))]
#[inline]
fn is_ascii(bytes: &[u8]) -> bool {
    let mut top = 0;
    #[cfg_attr(creusot, invariant(
        (top@ < 128) == forall<i: Int> 0 <= i && i < produced.len() ==> bytes@[i]@ < 128
    ))]
    for &byte in bytes {
        #[cfg(creusot)]
        proof_assert!(byte == bytes@[produced.len() - 1]);
        if byte > top {
            top = byte;
        }
    }
    top < 0x80
}

/// UTF-16 code units of the scalars that begin in `bytes[from..to]`. A `to`
/// inside a scalar counts that scalar whole, so the count never panics and
/// never decreases as `to` moves forward.
#[cfg_attr(creusot, requires(utf8_from(bytes@, from@)))]
#[cfg_attr(creusot, requires(from@ <= to@ && to@ <= bytes@.len()))]
#[cfg_attr(creusot, ensures(result@ == units(bytes@, from@, to@)))]
#[expect(
    clippy::manual_range_contains,
    reason = "Creusot 0.14 has no contract for Range::contains"
)]
#[inline]
pub(super) fn units_between(bytes: &[u8], from: usize, to: usize) -> usize {
    let Some(part) = bytes.get(from..to) else {
        return 0;
    };
    #[cfg(creusot)]
    proof_assert!(lead_bytes_count_the_units(bytes@, from@, to@); units_fit(bytes@, from@, to@); true);
    #[cfg(creusot)]
    proof_assert!(forall<i: Int> from@ <= i && i < to@ ==> bytes@[i] == part@[i - from@]);
    if is_ascii(part) {
        #[cfg(creusot)]
        proof_assert!(ascii_bytes_weigh_one_each(bytes@, from@, to@); true);
        return part.len();
    }
    let mut total = 0;
    #[cfg_attr(creusot, invariant(total@ == weights(bytes@, from@, from@ + produced.len())))]
    for &byte in part {
        #[cfg(creusot)]
        proof_assert!(byte == part@[produced.len() - 1]);
        #[cfg(creusot)]
        proof_assert!(byte == bytes@[from@ + produced.len() - 1]);
        #[cfg(creusot)]
        proof_assert!(weights_grow(bytes@, from@, from@ + produced.len(), to@); true);
        #[cfg(creusot)]
        proof_assert!(weights(bytes@, from@, from@ + produced.len()) <= bytes@.len() - from@);
        #[cfg(creusot)]
        proof_assert!(weights(bytes@, from@, from@ + produced.len()) == total@ + lead_weight(byte));
        if byte < 0x80 || byte >= 0xc0 {
            total += 1;
        }
        if byte >= 0xf0 {
            total += 1;
        }
    }
    total
}

/// The offset of the scalar boundary in `bytes[from..to]` that exactly
/// `column` UTF-16 code units of the range precede, or `None` when no
/// boundary has that many: the column falls between the halves of a
/// surrogate pair, or past `to`.
#[cfg_attr(creusot, requires(utf8_from(bytes@, from@) && boundary(bytes@, from@, to@)))]
#[cfg_attr(creusot, requires(from@ <= to@ && to@ <= bytes@.len()))]
#[cfg_attr(creusot, ensures(match result {
    Some(at) => from@ <= at@ && at@ <= to@ && boundary(bytes@, from@, at@)
        && units(bytes@, from@, at@) == column@,
    None => forall<at: Int> from@ <= at && at <= to@ && boundary(bytes@, from@, at)
        ==> units(bytes@, from@, at) != column@,
}))]
#[expect(
    clippy::manual_range_contains,
    reason = "Creusot 0.14 has no contract for Range::contains"
)]
#[inline]
pub(super) fn offset_of_units(
    bytes: &[u8],
    from: usize,
    to: usize,
    column: usize,
) -> Option<usize> {
    #[cfg(creusot)]
    proof_assert!(lead_bytes_count_the_units(bytes@, from@, to@); units_fit(bytes@, from@, to@);
        weights(bytes@, from@, to@) == units(bytes@, from@, to@)
            && units(bytes@, from@, to@) <= bytes@.len() - from@);
    #[cfg(creusot)]
    proof_assert!(forall<b: Int> from@ <= b && b <= to@ && boundary(bytes@, from@, b)
        ==> { units_fit_before_a_boundary(bytes@, from@, b); units(bytes@, from@, b) <= b - from@ });
    let line = bytes.get(from..to)?;
    // UTF-16 never needs more code units than UTF-8 needs bytes. Only inspect
    // the requested prefix: column zero must stay O(1) even on a huge line,
    // and Unicode later in the line cannot affect an ASCII prefix's offset.
    let prefix = line.get(..column)?;
    #[cfg(creusot)]
    proof_assert!(forall<i: Int> 0 <= i && i < column@ ==> prefix@[i] == bytes@[from@ + i]);
    if is_ascii(prefix) {
        #[cfg(creusot)]
        proof_assert!(forall<i: Int> from@ <= i && i < from@ + column@ ==> bytes@[i]@ < 128);
        #[cfg(creusot)]
        proof_assert!(ascii_bytes_walk_one_unit_each(bytes@, from@, from@ + column@);
            boundary(bytes@, from@, from@ + column@) && units(bytes@, from@, from@ + column@) == column@);
        return Some(from + column);
    }
    let mut seen = 0;
    let mut at = from;
    #[cfg_attr(creusot, invariant(at@ == from@ + produced.len()))]
    #[cfg_attr(creusot, invariant(seen@ == weights(bytes@, from@, at@)))]
    #[cfg_attr(creusot, invariant(forall<b: Int> from@ <= b && b < at@ && boundary(bytes@, from@, b)
        ==> units(bytes@, from@, b) < column@))]
    for &byte in line {
        #[cfg(creusot)]
        proof_assert!(byte == line@[produced.len() - 1]);
        #[cfg(creusot)]
        proof_assert!(byte == bytes@[at@]);
        #[cfg(creusot)]
        proof_assert!(weights_grow(bytes@, from@, at@ + 1, to@);
            weights(bytes@, from@, at@ + 1) <= weights(bytes@, from@, to@));
        #[cfg(creusot)]
        proof_assert!(boundaries_are_the_lead_bytes(bytes@, from@, at@);
            boundary(bytes@, from@, at@) == !is_trail(bytes@[at@]));
        if byte < 0x80 || byte >= 0xc0 {
            #[cfg(creusot)]
            proof_assert!(lead_bytes_count_the_units(bytes@, from@, at@);
                weights(bytes@, from@, at@) == units(bytes@, from@, at@));
            #[cfg(creusot)]
            proof_assert!(seen@ == units(bytes@, from@, at@));
            #[cfg(creusot)]
            proof_assert!(boundary(bytes@, from@, at@));
            if seen == column {
                return Some(at);
            }
            seen += if byte >= 0xf0 { 2 } else { 1 };
            if seen > column {
                #[cfg(creusot)]
                proof_assert!(a_lead_byte_weighs_its_units(bytes@, from@, at@);
                    lead_weight(byte) == scalar_units(byte));
                #[cfg(creusot)]
                proof_assert!(forall<b: Int> at@ < b && b <= to@ && boundary(bytes@, from@, b)
                    ==> { a_scalar_widens_the_units(bytes@, from@, at@, b); units(bytes@, from@, b) > column@ });
                return None;
            }
        }
        at += 1;
    }
    #[cfg(creusot)]
    proof_assert!(at@ == to@ && seen@ == units(bytes@, from@, to@));
    if seen == column { Some(at) } else { None }
}
