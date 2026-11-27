//! The text behind an [`super::EmitDocument`].
//!
//! A fragment, a piece assembled before it is appended elsewhere, is usually
//! a few bytes, and a `CompactString` keeps those inline without allocating.
//! A whole-output buffer takes thousands of appends, and there a plain heap
//! string is cheaper: an append is a capacity check and a copy inlined into
//! the emitter, where a `CompactString` append is an out-of-line call whose
//! inlining moves with unrelated code compiled next to it.

use core::fmt;
use core::ops::Deref;

use vize_l0::{String, string_builder::TextBuf};

/// The widest text a `CompactString` keeps without allocating.
const INLINE_CAPACITY: usize = core::mem::size_of::<String>();

#[derive(Clone)]
pub(super) enum Text {
    /// Short or handed-over text.
    Fragment(String),
    /// A buffer sized for a whole output.
    Buffer(TextBuf),
}

impl Text {
    /// Room for `capacity` bytes; a buffer once that cannot stay inline.
    pub(super) fn with_capacity(capacity: usize) -> Self {
        if capacity > INLINE_CAPACITY {
            Self::Buffer(TextBuf::with_capacity(capacity))
        } else {
            Self::Fragment(String::with_capacity(capacity))
        }
    }

    #[inline(always)]
    pub(super) fn push_str(&mut self, text: &str) {
        match self {
            Self::Fragment(fragment) => fragment.push_str(text),
            Self::Buffer(buffer) => buffer.push_str(text),
        }
    }

    #[inline(always)]
    pub(super) fn push(&mut self, ch: char) {
        match self {
            Self::Fragment(fragment) => fragment.push(ch),
            Self::Buffer(buffer) => buffer.push(ch),
        }
    }

    pub(super) fn insert_str(&mut self, at: usize, text: &str) {
        match self {
            Self::Fragment(fragment) => fragment.insert_str(at, text),
            Self::Buffer(buffer) => buffer.insert_str(at, text),
        }
    }

    pub(super) fn into_compact(self) -> String {
        match self {
            Self::Fragment(fragment) => fragment,
            Self::Buffer(buffer) => buffer.into_compact(),
        }
    }
}

impl Deref for Text {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        match self {
            Self::Fragment(fragment) => fragment,
            Self::Buffer(buffer) => buffer,
        }
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for Text {}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}
