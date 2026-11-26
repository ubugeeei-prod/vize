//! A growing UTF-8 buffer for generated text.
//!
//! Code builders assemble their output through many small appends. A
//! `CompactString` re-reads its inline-or-heap representation on every push,
//! and a byte vector needs an unchecked conversion at the end. This buffer is
//! a plain heap string instead: an append costs what a byte-vector append
//! costs, every append takes a `&str` or a `char`, so rustc's type checker
//! proves the text is well-formed UTF-8, and the finished heap buffer moves
//! into a [`CompactString`] without a copy.

use core::fmt;
use core::ops::Deref;

use compact_str::CompactString;

/// Generated text under construction. See the module documentation.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct TextBuf(std::string::String);

impl TextBuf {
    /// An empty buffer that has not allocated.
    #[inline]
    pub const fn new() -> Self {
        Self(std::string::String::new())
    }

    /// An empty buffer with room for `capacity` bytes.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(std::string::String::with_capacity(capacity))
    }

    /// Append `text`.
    #[inline]
    pub fn push_str(&mut self, text: &str) {
        self.0.push_str(text);
    }

    /// Append one scalar value.
    #[inline]
    pub fn push(&mut self, character: char) {
        self.0.push(character);
    }

    /// The text so far.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Reserve room for at least `additional` more bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    /// Keep the first `len` bytes. `len` must be a scalar boundary, such as a
    /// length this buffer had earlier; like [`String::truncate`], it panics
    /// otherwise.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }

    /// Insert `text` at byte `at`. `at` must be a scalar boundary, such as a
    /// length this buffer had earlier; like [`String::insert_str`], it panics
    /// otherwise.
    #[inline]
    pub fn insert_str(&mut self, at: usize, text: &str) {
        self.0.insert_str(at, text);
    }

    /// The finished text. A heap-sized buffer moves without copying.
    #[inline]
    pub fn into_compact(self) -> CompactString {
        CompactString::from(self.0)
    }
}

impl Deref for TextBuf {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TextBuf {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<TextBuf> for CompactString {
    #[inline]
    fn from(text: TextBuf) -> Self {
        text.into_compact()
    }
}

impl PartialEq<str> for TextBuf {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for TextBuf {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl fmt::Write for TextBuf {
    #[inline]
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0.push_str(text);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, character: char) -> fmt::Result {
        self.0.push(character);
        Ok(())
    }
}

impl fmt::Display for TextBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for TextBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::TextBuf;

    #[test]
    fn appends_insertions_and_truncation_keep_the_text() {
        let mut text = TextBuf::with_capacity(32);
        text.push_str("const é = ");
        let at = text.len();
        text.push('日');
        text.push_str(";\n");
        text.insert_str(at, "🦀");
        assert_eq!(text, "const é = 🦀日;\n");
        text.truncate(at);
        assert_eq!(text.as_str(), "const é = ");
    }

    #[test]
    fn a_heap_buffer_moves_into_a_compact_string() {
        let mut text = TextBuf::with_capacity(64);
        text.push_str("export default /*@__PURE__*/_defineComponent({})\n");
        let pointer = text.as_ptr();
        let compact = text.into_compact();
        assert_eq!(
            compact,
            "export default /*@__PURE__*/_defineComponent({})\n"
        );
        assert_eq!(compact.as_ptr(), pointer);
    }
}
