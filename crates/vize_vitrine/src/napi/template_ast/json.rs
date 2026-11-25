//! The JSON text buffer the template tree is written into.
//!
//! The whole tree crosses the N-API boundary as one string that `JSON.parse`
//! turns into objects, so the writer appends bytes directly instead of
//! building an intermediate value tree.

use vize_l1::markup::entity::{EntityContext, decode_one};

#[derive(Default)]
pub(super) struct Json {
    pub(super) bytes: Vec<u8>,
}

impl Json {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
        }
    }

    /// Append JSON syntax that needs no escaping.
    pub(super) fn raw(&mut self, text: &str) {
        self.bytes.extend_from_slice(text.as_bytes());
    }

    pub(super) fn number(&mut self, value: u32) {
        let mut digits = [0_u8; 10];
        let mut at = digits.len();
        let mut rest = value;
        loop {
            at -= 1;
            if let Some(digit) = digits.get_mut(at) {
                *digit = b'0' + (rest % 10) as u8;
            }
            rest /= 10;
            if rest == 0 {
                break;
            }
        }
        self.bytes
            .extend_from_slice(digits.get(at..).unwrap_or_default());
    }

    pub(super) fn boolean(&mut self, value: bool) {
        self.raw(if value { "true" } else { "false" });
    }

    /// Append `text` as a JSON string literal.
    pub(super) fn string(&mut self, text: &str) {
        self.bytes.push(b'"');
        self.escaped(text);
        self.bytes.push(b'"');
    }

    /// Append `text` with its character references decoded, as Vue's
    /// compiler decodes text and attribute values, as a JSON string literal.
    pub(super) fn decoded(&mut self, text: &str, context: EntityContext) {
        self.bytes.push(b'"');
        let bytes = text.as_bytes();
        let mut start = 0;
        let mut at = 0;
        while let Some(found) = bytes
            .get(at..)
            .and_then(|rest| rest.iter().position(|byte| *byte == b'&'))
        {
            let ampersand = at + found;
            let Some((entity, consumed)) = bytes
                .get(ampersand..)
                .and_then(|rest| decode_one(rest, context))
            else {
                at = ampersand + 1;
                continue;
            };
            self.escaped(text.get(start..ampersand).unwrap_or_default());
            entity.for_each(|character| {
                let mut buffer = [0_u8; 4];
                self.escaped(character.encode_utf8(&mut buffer));
            });
            at = ampersand + consumed;
            start = at;
        }
        self.escaped(text.get(start..).unwrap_or_default());
        self.bytes.push(b'"');
    }

    /// The minimal RFC 8259 escape set: `"`, `\` and control characters.
    /// Every other byte, multi-byte UTF-8 included, is copied verbatim.
    fn escaped(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let mut start = 0;
        for (at, byte) in bytes.iter().enumerate() {
            let escape: &[u8] = match byte {
                b'"' => b"\\\"",
                b'\\' => b"\\\\",
                b'\n' => b"\\n",
                b'\r' => b"\\r",
                b'\t' => b"\\t",
                0..=0x1f => b"",
                _ => continue,
            };
            self.bytes
                .extend_from_slice(bytes.get(start..at).unwrap_or_default());
            if escape.is_empty() {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let high = HEX.get(usize::from(byte >> 4)).copied().unwrap_or(b'0');
                let low = HEX.get(usize::from(byte & 0xf)).copied().unwrap_or(b'0');
                self.bytes
                    .extend_from_slice(&[b'\\', b'u', b'0', b'0', high, low]);
            } else {
                self.bytes.extend_from_slice(escape);
            }
            start = at + 1;
        }
        self.bytes
            .extend_from_slice(bytes.get(start..).unwrap_or_default());
    }
}
