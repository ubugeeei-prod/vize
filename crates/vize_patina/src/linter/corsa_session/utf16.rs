//! Byte offsets as the UTF-16 offsets the checker addresses.

/// Bytes between recorded positions in a non-ASCII source.
const STRIDE: usize = 256;

/// UTF-16 offsets of byte offsets in one source, for a batch of conversions.
///
/// Counting from the start of the source for every conversion made a file's
/// batch quadratic in its length. A conversion here counts from the nearest
/// recorded char boundary at or before the offset, and an ASCII source needs
/// no count at all.
pub(super) struct Utf16Offsets<'s> {
    source: &'s str,
    /// `(byte, utf16)` at the first char boundary at or after every
    /// [`STRIDE`] bytes. Empty for an ASCII source.
    checkpoints: Vec<(u32, u32)>,
}

impl<'s> Utf16Offsets<'s> {
    pub(super) fn new(source: &'s str) -> Self {
        let mut checkpoints = Vec::new();
        if !source.is_ascii() {
            checkpoints.reserve(source.len() / STRIDE + 1);
            let mut next = 0;
            let mut utf16 = 0u32;
            for (byte, character) in source.char_indices() {
                if byte >= next {
                    checkpoints.push((byte as u32, utf16));
                    next = byte + STRIDE;
                }
                utf16 += character.len_utf16() as u32;
            }
        }
        Self {
            source,
            checkpoints,
        }
    }

    /// The UTF-16 offset of the char boundary at or before `byte_offset`,
    /// clamped to the source.
    pub(super) fn get(&self, byte_offset: u32) -> u32 {
        let mut clamped = usize::min(byte_offset as usize, self.source.len());
        while clamped > 0 && !self.source.is_char_boundary(clamped) {
            clamped -= 1;
        }
        let index = self
            .checkpoints
            .partition_point(|&(byte, _)| byte as usize <= clamped);
        let Some(&(byte, utf16)) = index
            .checked_sub(1)
            .and_then(|index| self.checkpoints.get(index))
        else {
            // ASCII: one UTF-16 unit per byte.
            return clamped as u32;
        };
        let rest = self.source.get(byte as usize..clamped).unwrap_or_default();
        utf16 + rest.encode_utf16().count() as u32
    }
}

/// The reference conversion: the UTF-16 length of the prefix before the char
/// boundary at or before `byte_offset`.
#[cfg(test)]
pub(super) fn byte_offset_to_utf16_offset(source: &str, byte_offset: u32) -> u32 {
    let mut clamped = usize::min(byte_offset as usize, source.len());
    while clamped > 0 && !source.is_char_boundary(clamped) {
        clamped -= 1;
    }
    let prefix = source.get(..clamped).unwrap_or_default();
    prefix.encode_utf16().count() as u32
}

#[cfg(test)]
mod tests {
    use super::{Utf16Offsets, byte_offset_to_utf16_offset};

    #[test]
    fn batched_offsets_match_prefix_counts() {
        let mut mixed = String::new();
        for index in 0..300 {
            mixed.push_str(match index % 5 {
                0 => "plain ascii text ",
                1 => "日本語のテキスト",
                2 => "😀🎉",
                3 => "é\u{301}",
                _ => "\n",
            });
        }
        for source in ["", "ascii only", "a😀b", mixed.as_str()] {
            let offsets = Utf16Offsets::new(source);
            for byte in 0..=source.len() as u32 + 3 {
                assert_eq!(
                    offsets.get(byte),
                    byte_offset_to_utf16_offset(source, byte),
                    "{byte} in {} bytes",
                    source.len()
                );
            }
        }
    }
}
