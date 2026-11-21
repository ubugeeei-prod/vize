//! Global UTF-16 offsets for repeated queries on one immutable text.
use super::utf16_offset;
use crate::Span;

/// A successful conversion records one character boundary. Forward or
/// overlapping ranges can continue from it without indexing the whole file.
#[derive(Debug)]
pub struct Utf16Cursor<'s> {
    source: &'s str,
    byte: usize,
    units: u32,
    ascii_prefix: u32,
    scan_ascii: bool,
}

impl<'s> Utf16Cursor<'s> {
    #[must_use]
    #[inline]
    pub fn new(source: &'s str) -> Self {
        // Two vector-sized chunks cover small diagnostic documents without
        // making a request near the beginning scan an entire large file.
        let prefix = source.as_bytes().get(..64).unwrap_or(source.as_bytes());
        let ascii_prefix = if prefix.is_ascii() {
            prefix.len() as u32
        } else {
            0
        };
        Self {
            source,
            byte: 0,
            units: 0,
            ascii_prefix,
            scan_ascii: ascii_prefix != 0 && prefix.len() < source.len(),
        }
    }

    /// Convert a global UTF-16 range to a stage byte span, retaining exact
    /// half-open endpoints. An endpoint outside the 32-bit span space fails.
    #[inline]
    pub fn span(&mut self, start: u32, length: u32) -> Option<Span> {
        let end = start.checked_add(length)?;
        if end <= self.ascii_prefix || (self.scan_ascii && self.extend_ascii(end)) {
            return Some(Span::new(start, end));
        }
        let first = u32::try_from(self.offset(start)?).ok()?;
        let last = if length == 0 {
            first
        } else {
            u32::try_from(self.offset(end)?).ok()?
        };
        Some(Span::new(first, last))
    }

    // Confirm the next slice once. Adjacent diagnostics can then use direct
    // byte ranges, including for short ASCII files just beyond the first chunk.
    fn extend_ascii(&mut self, end: u32) -> bool {
        let bytes = self.source.as_bytes();
        if end as usize > bytes.len() {
            return false;
        }
        let start = self.ascii_prefix as usize;
        let limit = (end.saturating_add(255) as usize).min(bytes.len());
        let tail = &bytes[start..limit];
        if tail.is_ascii() {
            self.ascii_prefix = limit as u32;
            self.scan_ascii = limit < bytes.len();
        } else {
            self.scan_ascii = false;
            self.ascii_prefix += tail.iter().take_while(|byte| byte.is_ascii()).count() as u32;
        }
        end <= self.ascii_prefix
    }

    /// Convert a global UTF-16 offset to a UTF-8 byte offset. A point inside a
    /// surrogate pair or beyond the source fails and preserves the last boundary.
    pub fn offset(&mut self, units: u32) -> Option<usize> {
        if units == self.units {
            return Some(self.byte);
        }
        let byte = if units > self.units {
            let tail = self.source.get(self.byte..)?;
            self.byte + utf16_offset(tail, units - self.units)?
        } else {
            let distance = self.units - units;
            if distance < units {
                retreat(self.source.get(..self.byte)?, distance)?
            } else {
                utf16_offset(self.source, units)?
            }
        };
        self.units = units;
        self.byte = byte;
        Some(byte)
    }
}

fn retreat(text: &str, units: u32) -> Option<usize> {
    let byte = text.len().checked_sub(units as usize)?;
    if text.as_bytes().get(byte..)?.is_ascii() {
        return Some(byte);
    }
    let mut remaining = units;
    for (start, character) in text.char_indices().rev() {
        remaining = remaining.checked_sub(character.len_utf16() as u32)?;
        if remaining == 0 {
            return Some(start);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_cross_chunk_boundaries_and_recover_after_invalid_requests() {
        for marker in ["", "é", "😀", "雪"] {
            for prefix in [0, 63, 64, 65, 319, 320, 321] {
                let text = crate::cstr!("{}{marker}{}", "x".repeat(prefix), "y".repeat(320));
                let mut cursor = Utf16Cursor::new(&text);
                for at in [64, 320, 0, prefix, prefix + 1, prefix + 2, 650, 2] {
                    for len in [0, 1, 2, 64, 320, u32::MAX] {
                        let at = at as u32;
                        let expected = at.checked_add(len).and_then(|end| {
                            Some(Span::new(
                                u32::try_from(utf16_offset(&text, at)?).ok()?,
                                u32::try_from(utf16_offset(&text, end)?).ok()?,
                            ))
                        });
                        assert_eq!(
                            cursor.span(at, len),
                            expected,
                            "{prefix} {marker} {at} {len}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn every_order_and_invalid_boundary_agrees_with_the_original_conversion() {
        for text in ["", "ascii", "a\r\nb\u{2028}c", "雪🎨a😀é\0𠀀z"] {
            let limit = text.encode_utf16().count() as u32 + 2;
            for anchor in 0..=limit {
                let mut cursor = Utf16Cursor::new(text);
                for target in 0..=limit {
                    for at in [anchor, target, u32::MAX, target, anchor] {
                        assert_eq!(
                            cursor.offset(at),
                            utf16_offset(text, at),
                            "{text:?} {anchor} {target} {at}"
                        );
                    }
                }
            }
        }
    }
}
