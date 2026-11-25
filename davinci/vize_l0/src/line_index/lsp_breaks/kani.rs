#![allow(
    unexpected_cfgs,
    reason = "Kani selects the symbolic harness with cfg(kani)"
)]

/// The reference visits every line ending, including a trailing CR or LF.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(7)]
fn nearest_lsp_line_starts_match_every_short_byte_slice() {
    use super::{first_start, last_start};
    use crate::line_index::LineBreaks;

    let storage: [u8; 4] = kani::any();
    let len: usize = kani::any();
    kani::assume(len <= storage.len());
    let bytes = &storage[..len];
    let mut first = None;
    let mut last = None;
    for at in 0..len {
        if bytes[at] == b'\n' || (bytes[at] == b'\r' && bytes.get(at + 1) != Some(&b'\n')) {
            first.get_or_insert(at + 1);
            last = Some(at + 1);
        }
    }
    assert_eq!(first_start(bytes), first);
    assert_eq!(last_start(bytes), last);

    if let Ok(source) = core::str::from_utf8(bytes) {
        let mut starts = LineBreaks::Lsp.line_starts(source).skip(1);
        let head = starts.next();
        assert_eq!(head, first);
        assert_eq!(starts.last().or(head), last);
    }
}
