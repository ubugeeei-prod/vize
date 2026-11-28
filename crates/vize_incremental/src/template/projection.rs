//! Block-relative editor ranges projected into their authored source file.
//!
//! The caller obtains `body` from a `SourceBlock`'s `&str`, and that block
//! certifies that the base and the whole body fit in the original u32-sized
//! file. The byte kernel uses the standard UTF-8 boundary check directly,
//! so its bounds and arithmetic can be verified without a trusted str spec.

#[cfg(creusot)]
use creusot_std::prelude::*;
#[cfg(creusot)]
use davinci_unicode::{boundaries_are_the_lead_bytes, boundary, the_end_is_a_boundary, utf8_from};

/// Exactly the ordered ranges of whole UTF-8 forms inside this block.
#[cfg(creusot)]
#[logic(open(crate))]
pub(super) fn valid_range(body: Seq<u8>, start: Int, end: Int) -> bool {
    pearlite! {
        0 <= start && start <= end && end <= body.len()
            && boundary(body, 0, start) && boundary(body, 0, end)
    }
}

/// Shift a whole UTF-8 range by the block's authored base.
///
/// `body` is borrowed from a `SourceBlock`'s `&str`. The block also ensures
/// `base + body.len() <= u32::MAX`; this is the same caller obligation the
/// former `str::get` spelling used for its two additions.
#[cfg_attr(creusot, requires(utf8_from(body@, 0)))]
#[cfg_attr(creusot, requires(base@ + body@.len() <= u32::MAX@))]
#[cfg_attr(creusot, ensures(match result {
    Some((first, last)) => valid_range(body@, start@, end@)
        && first@ == base@ + start@ && last@ == base@ + end@,
    None => !valid_range(body@, start@, end@),
}))]
#[expect(
    clippy::manual_range_contains,
    reason = "Creusot 0.14 has no contract for Range::contains"
)]
#[inline]
pub(super) fn authored(body: &[u8], base: u32, start: u32, end: u32) -> Option<(u32, u32)> {
    let first = start as usize;
    let last = end as usize;
    if first > last || last > body.len() {
        return None;
    }
    #[cfg(creusot)]
    let _ = snapshot!(the_end_is_a_boundary(body@, 0));
    // At the end, both endpoints must be the end; neither byte is read.
    if first != body.len() {
        #[cfg(creusot)]
        let _ = snapshot!(boundaries_are_the_lead_bytes(body@, 0, first@));
        if first != 0 {
            let byte = *body.get(first)?;
            if byte >= 0x80 && byte < 0xc0 {
                return None;
            }
        }
        if last != body.len() {
            #[cfg(creusot)]
            let _ = snapshot!(boundaries_are_the_lead_bytes(body@, 0, last@));
            let byte = *body.get(last)?;
            if byte >= 0x80 && byte < 0xc0 {
                return None;
            }
        }
    }
    Some((base + start, base + end))
}
