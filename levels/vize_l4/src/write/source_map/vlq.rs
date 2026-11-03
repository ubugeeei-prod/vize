use vize_l0::String;

pub(super) const BASE64_CHARS: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Append the base64 VLQ encoding of a signed integer to `out`.
///
/// VLQ uses the low bit as the sign and 5-bit groups, the high (6th) bit of each
/// group being a continuation flag, matching the Source Map v3 encoding.
pub(super) fn encode_vlq(out: &mut String, value: i64) {
    // Zig-zag the sign into the low bit.
    let mut vlq: u64 = if value < 0 {
        ((-value as u64) << 1) | 1
    } else {
        (value as u64) << 1
    };

    loop {
        let mut digit = (vlq & 0b1_1111) as usize;
        vlq >>= 5;
        if vlq != 0 {
            // Set the continuation bit.
            digit |= 0b10_0000;
        }
        out.push(char::from(BASE64_CHARS.get(digit).copied().unwrap_or(b'A')));
        if vlq == 0 {
            break;
        }
    }
}
