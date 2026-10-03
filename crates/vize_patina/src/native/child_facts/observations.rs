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
    #[must_use]
    pub fn header_key(&self) -> u32 {
        self.header
    }
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
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
    #[must_use]
    pub fn header_key(&self) -> u32 {
        self.header
    }
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
}
