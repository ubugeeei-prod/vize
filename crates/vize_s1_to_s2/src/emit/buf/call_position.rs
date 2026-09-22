//! First call position of each helper alias in the emitted text.
//!
//! The helper-ordering comparator reads these positions many times per
//! module, so they are computed once, in a single scan over the hoists and
//! the body, instead of rescanning the whole module per alias per
//! comparison.

use oxc_syntax::identifier::is_identifier_part;

use super::super::helper::Helper;

/// First call offset of every helper alias across a sequence of text chunks
/// (the hoists, then the body), offsets running on across chunks.
pub(super) struct AliasPositions([usize; 64]);

impl AliasPositions {
    pub(super) fn scan<'s>(chunks: impl IntoIterator<Item = &'s str>) -> Self {
        let mut positions = [usize::MAX; 64];
        let mut offset = 0;
        for text in chunks {
            scan_chunk(text, offset, &mut positions);
            offset += text.len();
        }
        Self(positions)
    }

    pub(super) fn first(&self, helper: Helper) -> Option<usize> {
        let position = self.0[slot(helper)];
        (position != usize::MAX).then_some(position)
    }
}

fn slot(helper: Helper) -> usize {
    helper.bit().trailing_zeros() as usize
}

/// A call is an alias token (not preceded by an identifier character or
/// `.`) followed by optional whitespace and `(`, outside strings and
/// comments. Every alias is an ASCII identifier starting with `_`, so a
/// token is the maximal ASCII identifier run from that `_`; positions
/// inside the run can never start a call (an identifier character precedes
/// them), which is why the scan may jump to its end.
fn scan_chunk(text: &str, offset: usize, positions: &mut [usize; 64]) {
    let bytes = text.as_bytes();
    let mut position = 0;
    while position < bytes.len() {
        match bytes[position] {
            b'\'' | b'"' | b'`' => position = quoted_end(bytes, position),
            b'/' if bytes.get(position + 1) == Some(&b'/') => {
                position = line_comment_end(bytes, position);
            }
            b'/' if bytes.get(position + 1) == Some(&b'*') => {
                position = block_comment_end(bytes, position);
            }
            b'_' if token_may_start(text, position) => {
                let end = bytes[position..]
                    .iter()
                    .position(|byte| !is_ascii_identifier_part(*byte))
                    .map_or(bytes.len(), |length| position + length);
                if followed_by_call(bytes, end)
                    && let Some(helper) = helper_for_alias(&text[position..end])
                {
                    let first = &mut positions[slot(helper)];
                    if *first == usize::MAX {
                        *first = offset + position;
                    }
                }
                position = end;
            }
            _ => position += 1,
        }
    }
}

fn token_may_start(text: &str, position: usize) -> bool {
    text[..position]
        .chars()
        .next_back()
        .is_none_or(|ch| !is_identifier_part(ch) && ch != '.')
}

fn followed_by_call(bytes: &[u8], after: usize) -> bool {
    bytes[after..]
        .iter()
        .find(|byte| !byte.is_ascii_whitespace())
        == Some(&b'(')
}

const fn is_ascii_identifier_part(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn helper_for_alias(token: &str) -> Option<Helper> {
    Helper::ALL
        .into_iter()
        .find(|helper| helper.alias() == token)
}

fn line_comment_end(bytes: &[u8], position: usize) -> usize {
    bytes[position + 2..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |end| position + 2 + end)
}

fn block_comment_end(bytes: &[u8], position: usize) -> usize {
    bytes[position + 2..]
        .windows(2)
        .position(|pair| pair == b"*/")
        .map_or(bytes.len(), |end| position + 4 + end)
}

fn quoted_end(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut position = start + 1;
    while position < bytes.len() {
        match bytes[position] {
            b'\\' => position += 2,
            byte if byte == quote => return position + 1,
            _ => position += 1,
        }
    }
    bytes.len()
}

#[cfg(test)]
mod tests;
