use vize_l0::Span;

/// A closed, non-raw marker from one actual direct child. Content is opaque,
/// including zero-width, invalid JavaScript and entity-spelled expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeInterpolationFact {
    pub(super) header: u32,
    pub(super) span: Span,
    pub(super) content: Span,
}
impl NativeInterpolationFact {
    /// Returns the stable key of the authenticated parent header.
    #[must_use]
    pub fn header_key(&self) -> u32 {
        self.header
    }
    /// Returns the full original interpolation marker range.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
    /// Returns the original range inside the interpolation delimiters.
    #[must_use]
    pub fn content(&self) -> Span {
        self.content
    }
}

/// A counterexample derived from the genuine header and supplied marker only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeTextareaMustacheFact {
    pub(super) header: u32,
    pub(super) span: Span,
}
impl NativeTextareaMustacheFact {
    /// Returns the stable key of the authenticated textarea header.
    #[must_use]
    pub fn header_key(&self) -> u32 {
        self.header
    }
    /// Returns the original interpolation marker range.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
}
