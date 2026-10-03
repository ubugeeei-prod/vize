use vize_l0::{Span, String};
use vize_l1::markup::NativeLintTagKind;

/// Checked authored binding category, with opaque values left uninterpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeBindingKind {
    Static,
    Bind,
    Other,
}

/// Original selected tag grammar and ranges. Fields cannot be caller-authored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeHeaderFact {
    pub(super) tag: String,
    pub(super) span: Span,
    pub(super) opening: Span,
    pub(super) kind: NativeLintTagKind,
    pub(super) literal: bool,
}
impl NativeHeaderFact {
    #[must_use]
    pub fn tag(&self) -> &str {
        self.tag.as_str()
    }
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
    #[must_use]
    pub fn opening(&self) -> Span {
        self.opening
    }
    #[must_use]
    pub fn kind(&self) -> NativeLintTagKind {
        self.kind
    }
    #[must_use]
    pub fn header_is_literal(&self) -> bool {
        self.literal
    }
}

/// One checked original attribute, keyed by its actual header ordinal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeAttributeFact {
    pub(super) name: String,
    pub(super) span: Span,
    pub(super) kind: NativeBindingKind,
}
impl NativeAttributeFact {
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
    #[must_use]
    pub fn kind(&self) -> NativeBindingKind {
        self.kind
    }
}

/// Exact authored unsupported-ARIA counterexample, derived from both inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeUnsupportedAriaFact {
    pub(super) header: u32,
    pub(super) span: Span,
}
impl NativeUnsupportedAriaFact {
    #[must_use]
    pub fn header_key(&self) -> u32 {
        self.header
    }
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
}
