//! Exercise the public byte searches across aligned and unaligned SIMD chunks.
#![cfg(feature = "lsp-line-break-search")]

use vize_l0::line_index::lsp_breaks::{first_start, last_start};

fn endpoints(bytes: &[u8]) -> (Option<usize>, Option<usize>) {
    (first_start(bytes), last_start(bytes))
}

#[test]
fn nearest_breaks_at_vector_and_slice_boundaries() {
    let mut storage = [b'x'; 64 + 129];
    for offset in 0..64 {
        for len in [15, 16, 17, 31, 32, 33, 63, 64, 65, 129] {
            let bytes = &mut storage[offset..offset + len];
            assert_eq!(endpoints(bytes), (None, None));
            for left in 0..len {
                for &first in b"\r\n" {
                    bytes[left] = first;
                    assert_eq!(endpoints(bytes), (Some(left + 1), Some(left + 1)));
                    for right in left + 1..len {
                        for &last in b"\r\n" {
                            bytes[right] = last;
                            let head = if first == b'\r' && last == b'\n' && right == left + 1 {
                                right + 1
                            } else {
                                left + 1
                            };
                            assert_eq!(endpoints(bytes), (Some(head), Some(right + 1)));
                        }
                        bytes[right] = b'x';
                    }
                }
                bytes[left] = b'x';
            }
        }
    }
}
