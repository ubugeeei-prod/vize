//! The growing text of one virtual TypeScript projection.
//!
//! A projection grows to tens of kilobytes through thousands of small
//! appends. `CompactString` re-reads its inline-or-heap representation on
//! every push, which cost about twice the instructions of a `Vec`-backed
//! `String` per append on the generator's hot paths, so the projection buffer
//! is a plain heap string and only the finished text becomes a compact one.

#![expect(
    clippy::disallowed_types,
    reason = "a growing projection buffer is always heap-sized; std String appends are cheaper"
)]

use std::fmt;
use std::ops::Deref;

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct TextBuf(std::string::String);

impl TextBuf {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self(std::string::String::with_capacity(capacity))
    }

    pub(crate) fn from_text(text: &str) -> Self {
        Self(std::string::String::from(text))
    }

    #[inline]
    pub(crate) fn push_str(&mut self, text: &str) {
        self.0.push_str(text);
    }

    pub(crate) fn push_lib_text(&mut self, text: &str) {
        super::generated_names::push_lib_text(self, text);
    }

    #[inline]
    pub(crate) fn push(&mut self, character: char) {
        self.0.push(character);
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }

    #[cfg(test)]
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    /// The finished projection; a heap-sized buffer moves without copying.
    pub(crate) fn into_compact(self) -> vize_carton::String {
        vize_carton::String::from(self.0)
    }
}

impl Deref for TextBuf {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
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

impl std::ops::AddAssign<&str> for TextBuf {
    fn add_assign(&mut self, text: &str) {
        self.0.push_str(text);
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

impl vize_l0::ts_emit::TsBuf for TextBuf {
    #[inline]
    fn reserve_ts(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    #[inline]
    fn push_ts_str(&mut self, text: &str) {
        self.0.push_str(text);
    }
}

/// Text that projection helpers append to: the projection buffer or a small
/// compact string assembled for one fragment.
pub(crate) trait TsText: fmt::Write {
    fn push_str(&mut self, text: &str);
    fn push(&mut self, character: char);
}

impl TsText for TextBuf {
    #[inline]
    fn push_str(&mut self, text: &str) {
        self.0.push_str(text);
    }

    #[inline]
    fn push(&mut self, character: char) {
        self.0.push(character);
    }
}

impl TsText for vize_carton::String {
    #[inline]
    fn push_str(&mut self, text: &str) {
        vize_carton::String::push_str(self, text);
    }

    #[inline]
    fn push(&mut self, character: char) {
        vize_carton::String::push(self, character);
    }
}

impl From<TextBuf> for std::string::String {
    fn from(text: TextBuf) -> Self {
        text.0
    }
}

impl PartialEq<std::string::String> for TextBuf {
    fn eq(&self, other: &std::string::String) -> bool {
        self.0 == *other
    }
}

impl PartialEq<vize_carton::String> for TextBuf {
    fn eq(&self, other: &vize_carton::String) -> bool {
        self.0 == other.as_str()
    }
}

impl vize_l0::ts_emit::EmitLen for TextBuf {
    #[inline]
    fn hole_len(&self) -> usize {
        self.0.len()
    }
}

impl<B: vize_l0::ts_emit::TsBuf> vize_l0::ts_emit::EmitHole<B> for TextBuf {
    #[inline]
    fn emit_hole(&self, buffer: &mut B) {
        buffer.push_ts_str(&self.0);
    }
}
