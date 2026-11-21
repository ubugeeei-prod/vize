//! Packed line breaks for an immediately consumed short ASCII source prefix.
use super::Position;

#[derive(Clone, Copy, Default)]
struct Lines {
    breaks: u64,
    start: Position,
}

impl Lines {
    #[inline]
    fn at(self, offset: u32) -> Position {
        if offset == 0 {
            return self.start;
        }
        let before = self.breaks & (u64::MAX >> (64 - offset));
        if before == 0 {
            return Position::new(self.start.line, self.start.character + offset);
        }
        Position::new(
            self.start.line + before.count_ones(),
            offset - (64 - before.leading_zeros()),
        )
    }
}

/// Up to 256 ASCII bytes with LF line endings. Byte and UTF-16 columns agree.
/// A CR requires the caller's protocol line-ending policy.
pub struct ShortAsciiLines {
    groups: [Lines; 4],
    length: u32,
}

impl ShortAsciiLines {
    /// Classify the source and locate its line breaks together without allocating.
    #[inline]
    pub fn new(prefix: &str) -> Option<Self> {
        if prefix.len() > 256 {
            return None;
        }
        let mut groups = [Lines::default(); 4];
        let mut start = Position::default();
        for (group, bytes) in groups.iter_mut().zip(prefix.as_bytes().chunks(64)) {
            *group = Lines {
                breaks: masks(bytes)?,
                start,
            };
            start = group.at(bytes.len() as u32);
        }
        Some(Self {
            groups,
            length: prefix.len() as u32,
        })
    }

    /// Locate a byte, clamping at the end of the classified prefix.
    #[inline]
    pub fn position(&self, byte: u32) -> Position {
        let at = byte.min(self.length);
        if at == 0 {
            return Position::default();
        }
        let index = (at - 1) / 64;
        let (_, tail) = self.groups.split_at(index as usize);
        let [group, ..] = tail else {
            return Position::default();
        };
        group.at(at - index * 64)
    }
}

#[inline]
fn scalar(bytes: &[u8]) -> Option<u64> {
    let mut mask = 0;
    for (index, &byte) in bytes.iter().enumerate() {
        if !byte.is_ascii() || byte == b'\r' {
            return None;
        }
        mask |= u64::from(byte == b'\n') << index;
    }
    Some(mask)
}

#[cfg(not(target_arch = "x86_64"))]
#[inline]
fn masks(bytes: &[u8]) -> Option<u64> {
    scalar(bytes)
}

#[cfg(target_arch = "x86_64")]
#[inline]
fn masks(bytes: &[u8]) -> Option<u64> {
    use core::arch::x86_64::{
        _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8,
    };
    #[inline]
    fn lanes(block: &[u8; 16]) -> (u32, u32) {
        // SAFETY: SSE2 is part of x86-64. A complete 16-byte array backs
        // the unaligned load, including the overlapping final load below.
        unsafe {
            let v = _mm_loadu_si128(block.as_ptr().cast());
            let lf = _mm_cmpeq_epi8(v, _mm_set1_epi8(b'\n' as i8));
            let cr = _mm_cmpeq_epi8(v, _mm_set1_epi8(b'\r' as i8));
            (
                _mm_movemask_epi8(lf) as u32,
                _mm_movemask_epi8(_mm_or_si128(cr, v)) as u32,
            )
        }
    }
    if bytes.len() < 16 {
        return scalar(bytes);
    }
    let (blocks, tail) = bytes.as_chunks::<16>();
    let mut result = 0;
    let mut invalid = 0;
    for (index, block) in blocks.iter().enumerate() {
        let (lf, other) = lanes(block);
        result |= u64::from(lf) << (index * 16);
        invalid |= other;
    }
    if !tail.is_empty() {
        let (lf, other) = lanes(bytes.last_chunk::<16>()?);
        let drop = 16 - tail.len();
        result |= u64::from(lf >> drop) << (blocks.len() * 16);
        invalid |= other >> drop;
    }
    (invalid == 0).then_some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line_index::offset_to_position;

    #[test]
    fn every_prefix_and_line_crossing_matches_the_original_coordinate() {
        let source = "a\nbcdefghijklmn\n\n".repeat(16);
        for end in 0..=256 {
            let (prefix, _) = source.split_at(end);
            let Some(index) = ShortAsciiLines::new(prefix) else {
                panic!("short ASCII prefix")
            };
            for at in 0..=end + 1 {
                assert_eq!(index.position(at as u32), offset_to_position(prefix, at));
            }
        }
        for offset in 0..=255 {
            for suffix in ["\r", "雪"] {
                let input = crate::cstr!("{}{suffix}", "x".repeat(offset));
                assert!(ShortAsciiLines::new(&input).is_none());
            }
        }
        assert!(ShortAsciiLines::new(&"x".repeat(257)).is_none());
    }
}
