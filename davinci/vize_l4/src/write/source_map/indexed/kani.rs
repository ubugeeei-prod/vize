mod geometry;
use geometry::{STRIDE, checkpoint, prefix_units};

#[kani::proof]
fn checkpoint_contains_every_offset() {
    let byte: usize = kani::any();
    let start = checkpoint(byte);
    assert_eq!(start % STRIDE, 0);
    assert!(start <= byte && byte - start < STRIDE);
    assert_eq!(start / STRIDE, byte / STRIDE);
}

#[kani::proof]
fn every_checkpoint_preserves_utf16_length() {
    let prefix_bytes: usize = kani::any();
    let prefix_utf16: usize = kani::any();
    let tail_bytes: usize = kani::any();
    let tail_utf16: usize = kani::any();
    kani::assume(prefix_utf16 <= prefix_bytes && tail_utf16 <= tail_bytes);
    kani::assume(prefix_bytes.checked_add(tail_bytes).is_some());
    let actual = prefix_units(prefix_bytes, prefix_bytes - prefix_utf16, tail_utf16);
    assert_eq!(actual, prefix_utf16 + tail_utf16);
    assert!(actual <= prefix_bytes + tail_bytes);
}
