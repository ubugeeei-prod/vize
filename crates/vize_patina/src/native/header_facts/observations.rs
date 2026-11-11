use vize_l0::{Span, String};
use vize_l1::markup::NativeLintTagKind;

/// Checked authored binding category, with opaque values left uninterpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeBindingKind {
    /// A plain authored attribute with a literal value.
    Static,
    /// A `v-bind` attribute with a checked static argument.
    Bind,
    /// Another supported directive category whose value remains opaque.
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
    /// Returns the original authored tag name.
    #[must_use]
    pub fn tag(&self) -> &str {
        self.tag.as_str()
    }
    /// Returns the full original element-header range.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
    /// Returns the original opening-tag range.
    #[must_use]
    pub fn opening(&self) -> Span {
        self.opening
    }
    /// Returns the producer-classified lint tag kind.
    #[must_use]
    pub fn kind(&self) -> NativeLintTagKind {
        self.kind
    }
    /// Reports whether the header belongs to a literal `v-pre` context.
    #[must_use]
    pub fn header_is_literal(&self) -> bool {
        self.literal
    }
}

/// One checked original attribute, keyed by its actual header ordinal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeAttributeFact {
    pub(super) name: String,
    pub(super) value: Option<String>,
    pub(super) span: Span,
    pub(super) kind: NativeBindingKind,
}
impl NativeAttributeFact {
    /// Returns the checked authored attribute or binding name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    /// Returns the original literal value for a static attribute.
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
    /// Returns the original full attribute range.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
    /// Returns the producer-classified binding category.
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
    /// Returns the stable key of the authenticated element header.
    #[must_use]
    pub fn header_key(&self) -> u32 {
        self.header
    }
    /// Returns the original unsupported attribute range.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
}
