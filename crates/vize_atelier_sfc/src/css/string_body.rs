//! The body of a CSS string token, one code point at a time.
//!
//! CSS Syntax Level 3, "Consume a string token": after the opening quote,
//! the string ends at the next code point equal to that quote. A REVERSE
//! SOLIDUS (U+005C) escapes the code point after it, whatever it is, so an
//! escaped quote or an escaped backslash never ends the string. Every other
//! code point is part of the string. The scoped CSS scanners copy string
//! bodies verbatim and only need to know where each one ends.
//!
//! `kani.rs` beside this file checks [`StringBody::step`] against that
//! statement for every Unicode scalar.

/// Where a scanner stands inside a CSS string, after its opening quote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StringBody {
    quote: char,
    escaped: bool,
}

impl StringBody {
    /// The body that the quote `quote` opens.
    #[inline]
    pub(crate) const fn open(quote: char) -> Self {
        Self {
            quote,
            escaped: false,
        }
    }

    /// The body after the code point `c`, or `None` when `c` ends the string.
    #[inline]
    pub(crate) fn step(self, c: char) -> Option<Self> {
        if self.escaped {
            Some(Self::open(self.quote))
        } else if c == '\\' {
            Some(Self {
                quote: self.quote,
                escaped: true,
            })
        } else if c == self.quote {
            None
        } else {
            Some(self)
        }
    }
}
