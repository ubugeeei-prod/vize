//! Kani checks the shipped Source Map v3 VLQ writer (`vlq.rs`) against the
//! decoder the Source Map v3 format defines: a value is a run of base64
//! digits, each carrying five payload bits least-significant group first, bit
//! 5 set on every digit but the last, and the payload's low bit is the sign.
//! The decoder below is written from that statement and the RFC 4648 base64
//! alphabet, not from the encoder or its table.
//!
//! The writer's loop runs once per five-bit group, at most 13 times for a
//! 64-bit payload, so an unwind of 15 covers every `i64` the writer accepts:
//! the check is exhaustive over that domain, not a sample. `i64::MIN` is
//! outside it (its magnitude has no `i64`), and the mappings writer never
//! passes it: every delta is a difference of two `u32` coordinates.
//!
//! `vlq.rs` is compiled exactly as shipped. The root stands in for `vize_l0`
//! with a `String` that records the characters the writer pushes, which is
//! all the writer does with the product's `CompactString`.

extern crate self as vize_l0;

/// The pushed characters, in order.
pub struct String {
    chars: [char; 16],
    len: usize,
}

impl String {
    pub fn push(&mut self, ch: char) {
        assert!(
            self.len < self.chars.len(),
            "more digits than a 64-bit payload needs"
        );
        self.chars[self.len] = ch;
        self.len += 1;
    }
}

mod vlq;

#[cfg(kani)]
mod proofs {
    use super::String;
    use super::vlq::encode_vlq;

    /// RFC 4648 base64 digit value, or `None` outside the alphabet.
    fn base64_digit(ch: char) -> Option<u64> {
        match ch {
            'A'..='Z' => Some(u64::from(ch) - u64::from('A')),
            'a'..='z' => Some(u64::from(ch) - u64::from('a') + 26),
            '0'..='9' => Some(u64::from(ch) - u64::from('0') + 52),
            '+' => Some(62),
            '/' => Some(63),
            _ => None,
        }
    }

    #[kani::proof]
    #[kani::unwind(15)]
    fn vlq_digits_decode_to_the_encoded_value() {
        let value: i64 = kani::any();
        kani::assume(value != i64::MIN);
        let mut out = String {
            chars: ['\0'; 16],
            len: 1,
        };
        out.chars[0] = ';';
        encode_vlq(&mut out, value);
        // Appending leaves what was already written untouched.
        assert_eq!(out.chars[0], ';');
        let digits = out.chars.get(1..out.len).unwrap_or_default();
        // A 64-bit payload needs at most ceil(64 / 5) groups.
        assert!(!digits.is_empty() && digits.len() <= 13);

        let mut payload: u64 = 0;
        for (group, &ch) in digits.iter().enumerate() {
            let Some(digit) = base64_digit(ch) else {
                panic!("a VLQ digit outside the base64 alphabet");
            };
            let last = group + 1 == digits.len();
            // The continuation bit is set exactly on every digit but the last.
            assert_eq!(digit & 0b10_0000 == 0, last);
            let bits = digit & 0b1_1111;
            let shift = 5 * group as u32;
            // No payload bit falls off the 64-bit end.
            assert_eq!((bits << shift) >> shift, bits);
            // The encoding is minimal: no trailing all-zero group.
            assert!(!last || group == 0 || bits != 0);
            payload |= bits << shift;
        }
        // Zero is never written as negative zero.
        assert_ne!(payload, 1);
        let magnitude = (payload >> 1) as i64;
        let decoded = if payload & 1 == 1 {
            -magnitude
        } else {
            magnitude
        };
        assert_eq!(decoded, value);

        kani::cover!(value == 0, "zero");
        kani::cover!(value == -1, "negative one");
        kani::cover!(digits.len() == 1 && value == 15, "largest single digit");
        kani::cover!(digits.len() == 2 && value == 16, "first two-digit value");
        kani::cover!(digits.len() == 13 && value == i64::MAX, "largest value");
        kani::cover!(digits.len() == 13 && value == -i64::MAX, "smallest value");
    }
}
