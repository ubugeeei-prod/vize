//! Lowercase hexadecimal rendering of SHA-256 digests.

use vize_s0::String;

/// `bytes` as lowercase hexadecimal, two characters per byte.
pub(crate) fn lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        for nibble in [byte >> 4, byte & 0x0f] {
            if let Some(digit) = char::from_digit(u32::from(nibble), 16) {
                output.push(digit);
            }
        }
    }
    output
}
