//! Check the original literal writer on every ASCII scalar. Eight bytes suffice
//! for the two quotes and the longest escape produced for one scalar.

#[derive(Default)]
struct String {
    bytes: u64,
    len: u8,
}

impl String {
    fn append(&mut self, byte: u8) {
        assert!(self.len < 8);
        self.bytes = (self.bytes << 8) | u64::from(byte);
        self.len += 1;
    }

    fn push(&mut self, value: char) {
        assert!(value.is_ascii());
        self.append(value as u8);
    }

    fn push_str(&mut self, value: &str) {
        for byte in value.bytes() {
            self.append(byte);
        }
    }
}

mod js_string;

fn nibble(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    }
}

fn decode(output: String) -> Option<u8> {
    match (output.len, output.bytes.to_be_bytes()) {
        (3, [0, 0, 0, 0, 0, b'"', raw, b'"']) if raw >= 32 && !matches!(raw, b'"' | b'\\') => {
            Some(raw)
        }
        (4, [0, 0, 0, 0, b'"', b'\\', tag, b'"']) => match tag {
            b'"' | b'\\' => Some(tag),
            b'b' => Some(8),
            b't' => Some(9),
            b'n' => Some(10),
            b'f' => Some(12),
            b'r' => Some(13),
            _ => None,
        },
        (8, [b'"', b'\\', b'u', b'0', b'0', high, low, b'"']) => {
            Some(nibble(high)? * 16 + nibble(low)?)
        }
        _ => None,
    }
}

#[kani::proof]
#[kani::unwind(9)]
fn compiler_emits_original_ascii() {
    let byte = kani::any::<u8>() & 0x7f;
    let input = [byte];
    let mut output = String::default();
    js_string::push_js_string_literal(&mut output, core::str::from_utf8(&input).unwrap());
    assert_eq!(decode(output), Some(byte));
}
