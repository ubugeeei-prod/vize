//! Repeated underline characters from the immutable renderer kernel.
use std::fmt;

pub(super) struct Underline {
    pub(super) padding: usize,
    pub(super) left: usize,
    pub(super) marker: char,
    pub(super) right: usize,
    pub(super) line: char,
}

impl Underline {
    pub(super) fn write(&self, f: &mut impl fmt::Write) -> fmt::Result {
        // Use pre-encoded chunks for built-in theme characters.
        let Some((underline_chunk, char_len)) = (match self.line {
            '─' => Some((UNICODE_BARS, '─'.len_utf8())),
            '^' => Some((ASCII_CARETS, 1)),
            _ => None,
        }) else {
            write_repeated_char(f, ' ', self.padding)?;
            write_repeated_char(f, self.line, self.left)?;
            f.write_char(self.marker)?;
            return write_repeated_char(f, self.line, self.right);
        };

        write_repeated_chunk(f, SPACES, 1, self.padding)?;
        write_repeated_chunk(f, underline_chunk, char_len, self.left)?;
        f.write_char(self.marker)?;
        write_repeated_chunk(f, underline_chunk, char_len, self.right)
    }
}

const CHUNK_CHARS: usize = 64;
const MIN_CHUNKED_CHARS: usize = 8;
const SPACES: &str = concat!(
    "                                ",
    "                                "
);
const UNICODE_BARS: &str = concat!(
    "────────────────────────────────",
    "────────────────────────────────"
);
const ASCII_CARETS: &str = concat!(
    "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^",
    "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^"
);

fn write_repeated_chunk(
    f: &mut impl fmt::Write,
    chunk: &str,
    char_len: usize,
    mut count: usize,
) -> fmt::Result {
    while count > CHUNK_CHARS {
        f.write_str(chunk)?;
        count -= CHUNK_CHARS;
    }
    if count == 0 {
        Ok(())
    } else {
        f.write_str(&chunk[..count * char_len])
    }
}

pub(super) fn write_repeated_char(f: &mut impl fmt::Write, c: char, count: usize) -> fmt::Result {
    for _ in 0..count {
        f.write_char(c)?;
    }
    Ok(())
}

#[inline]
pub(super) fn write_padding(f: &mut impl fmt::Write, count: usize) -> fmt::Result {
    if count < MIN_CHUNKED_CHARS {
        write_repeated_char(f, ' ', count)
    } else {
        write_repeated_chunk(f, SPACES, 1, count)
    }
}

impl fmt::Display for Underline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}
