//! The text behind an [`super::EmitDocument`].
//!
//! A fragment, a piece assembled before it is appended elsewhere, is usually
//! a few bytes, and a `CompactString` keeps those inline without allocating.
//! Text that outgrows the inline bytes moves to a plain heap string: there an
//! append is a capacity check and a copy inlined into the emitter, where a
//! `CompactString` append is an out-of-line call whose inlining moves with
//! unrelated code compiled next to it.

use core::fmt;
use core::ops::Deref;

use vize_l0::{String, string_builder::TextBuf};

/// The widest text a `CompactString` keeps without allocating.
const INLINE_CAPACITY: usize = core::mem::size_of::<String>();

/// The smallest heap allocation a `CompactString` makes.
const MIN_HEAP_CAPACITY: usize = INLINE_CAPACITY + core::mem::size_of::<usize>();

/// The capacity a `CompactString` of `len` bytes grows to for `additional`
/// more. Buffers grow the same way, so every allocation keeps the size the
/// `CompactString` text made and the allocator sees the same requests.
#[inline(always)]
fn amortized_capacity(len: usize, additional: usize) -> usize {
    (len.saturating_mul(3) / 2)
        .max(len.saturating_add(additional))
        .max(MIN_HEAP_CAPACITY)
}

#[derive(Clone)]
pub(super) enum Text {
    /// Inline or handed-over text.
    Fragment(String),
    /// Text that needs the heap.
    Buffer(TextBuf),
}

impl Text {
    /// Room for `capacity` bytes; a buffer once that cannot stay inline.
    pub(super) fn with_capacity(capacity: usize) -> Self {
        if capacity > INLINE_CAPACITY {
            Self::Buffer(TextBuf::with_capacity(capacity.max(MIN_HEAP_CAPACITY)))
        } else {
            Self::Fragment(String::with_capacity(capacity))
        }
    }

    #[inline(always)]
    pub(super) fn push_str(&mut self, text: &str) {
        match self {
            Self::Buffer(buffer) => {
                reserve(buffer, text.len());
                buffer.push_str(text);
            }
            Self::Fragment(fragment) if fragment.len() + text.len() <= INLINE_CAPACITY => {
                fragment.push_str(text);
            }
            Self::Fragment(_) => self.promote(text),
        }
    }

    #[inline(always)]
    pub(super) fn push(&mut self, ch: char) {
        match self {
            Self::Buffer(buffer) => {
                reserve(buffer, ch.len_utf8());
                buffer.push(ch);
            }
            Self::Fragment(fragment) => {
                let mut bytes = [0; 4];
                let text = ch.encode_utf8(&mut bytes);
                if fragment.len() + text.len() <= INLINE_CAPACITY {
                    fragment.push_str(text);
                } else {
                    self.promote(text);
                }
            }
        }
    }

    /// Move a fragment that `text` would push onto the heap into a buffer,
    /// then append `text`. A heap fragment moves without copying.
    #[inline(never)]
    fn promote(&mut self, text: &str) {
        let Self::Fragment(fragment) = self else {
            return;
        };
        let mut buffer = if fragment.is_heap_allocated() {
            TextBuf::from(core::mem::take(fragment))
        } else {
            let capacity = amortized_capacity(fragment.len(), text.len());
            let mut buffer = TextBuf::with_capacity(capacity);
            buffer.push_str(fragment);
            buffer
        };
        reserve(&mut buffer, text.len());
        buffer.push_str(text);
        *self = Self::Buffer(buffer);
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

/// Make room for `additional` bytes, growing like a `CompactString`.
#[inline(always)]
fn reserve(buffer: &mut TextBuf, additional: usize) {
    if buffer.capacity() - buffer.len() < additional {
        grow(buffer, additional);
    }
}

#[cold]
#[inline(never)]
fn grow(buffer: &mut TextBuf, additional: usize) {
    let len = buffer.len();
    buffer.reserve_exact(amortized_capacity(len, additional) - len);
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
