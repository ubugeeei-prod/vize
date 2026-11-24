//! Model-check the piece of `rebuild_source` that its `unsafe` rests on:
//! `fill`, which copies the rebuild into the string's spare capacity before
//! `set_len(written)` publishes it.
//!
//! Laws (the SAFETY comment's premises): `fill` reports at most the spare
//! tail's length; the reported bytes equal a prefix of whole pieces of the
//! rebuild rule (static text verbatim, `{{ text }}`, `{{{ text }}}`), so each
//! of them was written (Kani gives an unwritten slot every value, and one of
//! them differs), and they are valid UTF-8: a concatenation of whole `&str`
//! pieces is valid UTF-8 because each piece is. A tail as long as the whole
//! spelling, which is what `rebuild_source` reserves, receives all of it.
//!
//! Bounds: two parts with the texts `é` and `日` (two and three bytes), each of
//! any kind, and a spare tail of every length up to the whole spelling (5..=21
//! bytes). `fill` copies bytes without reading them, so the piece lengths are
//! what its arithmetic depends on. `CompactString`'s own `spare_capacity_mut`
//! and `set_len` stay outside the model: CBMC exceeded 8 GB on them, so they
//! rest on `compact_str`'s contract and the `text_rebuild` fuzz target.
#![allow(
    unexpected_cfgs,
    reason = "Kani selects its symbolic harnesses with cfg(kani)"
)]

#[cfg(kani)]
use super::{TextPart, delimiters, fill};
#[cfg(kani)]
use core::mem::MaybeUninit;
#[cfg(kani)]
use vize_l0::{Span, String};

#[cfg(kani)]
const TEXTS: [&str; 2] = ["é", "日"];

#[cfg(kani)]
const WHOLE: usize = 21;

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(22)]
#[kani::solver(cadical)]
fn fill_writes_whole_utf8_pieces_within_the_spare_tail() {
    let parts = TEXTS.map(|text| {
        let dynamic: bool = kani::any();
        TextPart {
            text: String::from(text),
            span: Span::new(0, 0),
            dynamic,
            raw_html: dynamic && kani::any::<bool>(),
        }
    });
    // The specified spelling and the end offset of each of its six pieces.
    let mut spelling = [0u8; WHOLE];
    let mut ends = [0usize; 7];
    let mut len = 0;
    for (index, part) in parts.iter().enumerate() {
        let (open, close) = delimiters(part);
        for (offset, piece) in [open, part.text.as_str(), close].iter().enumerate() {
            spelling[len..len + piece.len()].copy_from_slice(piece.as_bytes());
            len += piece.len();
            ends[1 + index * 3 + offset] = len;
        }
    }
    let mut buffer = [MaybeUninit::<u8>::uninit(); WHOLE];
    let tail: usize = kani::any();
    kani::assume(tail <= len);
    let written = fill(&mut buffer[..tail], &parts);
    assert!(written <= tail);
    let whole_pieces = ends.map(|end| end == written);
    assert!(whole_pieces != [false; 7]);
    // SAFETY: `written <= tail <= WHOLE`; the bytes are compared with the
    // specification, so an unwritten slot fails the comparison below.
    let bytes = unsafe { core::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), written) };
    assert!(bytes == &spelling[..written]);
    if tail == len {
        assert_eq!(written, len);
    }
    kani::cover!(written < tail, "a piece that does not fit stops the copy");
    kani::cover!(written == WHOLE, "two raw-HTML parts, whole");
    kani::cover!(len == 5 && written == 5, "two static parts, whole");
}
