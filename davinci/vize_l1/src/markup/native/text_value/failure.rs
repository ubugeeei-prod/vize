use super::{NativeChild, NativeTextValueError};
use crate::{SurfaceChild, Token};
use vize_l0::{SourceBlock, Span};

/// Owned typed preparation refusal with the actual child's full source/token.
/// The selected Component continues to own its surface and parser diagnostics.
/// These neutral observations cannot be joined as an admitted text value.
#[derive(Debug)]
pub struct NativeTextValueFailure<'a> {
    kind: NativeTextValueError,
    block: SourceBlock<'a>,
    ordinal: usize,
    token: Option<Token<'a>>,
    parent_tag: Option<&'a str>,
}
impl<'a> NativeTextValueFailure<'a> {
    pub(super) fn original(kind: NativeTextValueError, child: &NativeChild<'_, 'a>) -> Self {
        Self {
            kind,
            block: child.component().block(),
            ordinal: child.ordinal(),
            token: match child.surface() {
                SurfaceChild::Text(token) => Some(*token),
                _ => None,
            },
            parent_tag: child.parent_element().map(crate::Element::tag),
        }
    }
    #[must_use]
    pub const fn kind(&self) -> NativeTextValueError {
        self.kind
    }
    /// Foreign-child refusal retains that child's actual original source.
    #[must_use]
    pub const fn block(&self) -> SourceBlock<'a> {
        self.block
    }
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    #[must_use]
    pub const fn token(&self) -> Option<&Token<'a>> {
        self.token.as_ref()
    }
    #[must_use]
    pub const fn parent_tag(&self) -> Option<&'a str> {
        self.parent_tag
    }
    #[must_use]
    pub fn span(&self) -> Option<Span> {
        self.token.and_then(|token| self.block.span_of(token.text))
    }
}
