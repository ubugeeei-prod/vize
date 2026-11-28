//! Deductive editor projection laws on the exact kernel the product uses.
//!
//! This is the library root of `tests/formal/creusot/editor-projection`.
//! Ordinary module discovery includes `projection.rs`, also compiled by
//! `vize_incremental`; no copy or alternate implementation is proved.
//! The `SourceBlock` caller lends valid UTF-8 bytes and supplies the file
//! size bound. `davinci_unicode` defines scalar boundaries independently
//! and proves that they are exactly the lead bytes or the end of the text.
//!
//! This candidate still needs generated certificates and successful replay.

mod projection;

use creusot_std::prelude::*;
use davinci_unicode::{boundary, utf8_from};

/// For every block length and u32 base/endpoints, accept exactly the ordered
/// scalar-boundary ranges and return their exact file-absolute endpoints.
#[requires(utf8_from(body@, 0))]
#[requires(base@ + body@.len() <= u32::MAX@)]
#[ensures(match result {
    Some((first, last)) => start@ <= end@ && end@ <= body@.len()
        && boundary(body@, 0, start@) && boundary(body@, 0, end@)
        && first@ == base@ + start@ && last@ == base@ + end@,
    None => !(start@ <= end@ && end@ <= body@.len()
        && boundary(body@, 0, start@) && boundary(body@, 0, end@)),
})]
pub fn all_utf8_ranges_match_the_reference_and_fit_the_source_file(
    body: &[u8],
    base: u32,
    start: u32,
    end: u32,
) -> Option<(u32, u32)> {
    projection::authored(body, base, start, end)
}
