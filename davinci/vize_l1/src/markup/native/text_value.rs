//! Original text source preparation, without whitespace or target admission.

use super::{NativeChild, NativeTemplateComponent};
use crate::{Token, embed::EmbedSource};
use vize_l0::Span;

mod failure;
mod origin;
pub use failure::NativeTextValueFailure;
use origin::Origin;

/// Original membership or preparation refusal; the original tree stays owned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTextValueError {
    ForeignComponent,
    NotText,
    RecoveredComponent,
    Verbatim,
    RawTextParent,
    RcDataParent,
    Source(crate::embed::SourceError),
}

/// Normally owned original text token and its complete General/Text-context
/// source preparation. This establishes neither the lexer's actual Data mode,
/// Vue whitespace semantics nor File completion.
///
/// ```compile_fail
/// use vize_l1::markup::NativeTextValue;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeTextValue<'static>>();
/// ```
/// ```compile_fail
/// use vize_l1::{Token, embed::EmbedSource, markup::NativeTextValue};
/// fn forge(source: EmbedSource<'static>) -> NativeTextValue<'static> {
///     NativeTextValue {
///         origin: panic!(), original: Token::present("", source.text()), source,
///     }
/// }
/// ```
/// ```compile_fail
/// use vize_l1::markup::{NativeChild, NativeComponent};
/// fn from_unselected(owner: &NativeComponent<'static>, child: NativeChild<'_, 'static>) {
///     owner.observe_text_value(child);
/// }
/// ```
pub struct NativeTextValue<'a> {
    origin: Origin<'a>,
    original: Token<'a>,
    source: EmbedSource<'a>,
}

impl<'a> NativeTemplateComponent<'a> {
    /// Call existing General/Text source preparation once at this original child.
    /// The lexer may already have decoded callbacks when constructing its raw
    /// surface; this is not a claim of one decoder call across the whole parse.
    /// No root selection, sibling iteration or JavaScript parse occurs.
    /// Raw-text/RCDATA parents and inherited verbatim mode remain refused.
    pub fn observe_text_value(
        &self,
        child: NativeChild<'_, 'a>,
    ) -> Result<NativeTextValue<'a>, NativeTextValueFailure<'a>> {
        let prepare = || {
            let (origin, original) = Origin::original(self, &child)?;
            let source = crate::embed::source::prepare_text_value(
                self.component().allocator(),
                origin.block.root_source(),
                origin.span,
            )
            .map_err(NativeTextValueError::Source)?;
            Ok(NativeTextValue {
                origin,
                original,
                source,
            })
        };
        prepare().map_err(|kind| NativeTextValueFailure::original(kind, &child))
    }
}

impl<'a> NativeTextValue<'a> {
    /// Neutral original token observations, including leading bytes and holes.
    #[must_use]
    pub const fn token(&self) -> &Token<'a> {
        &self.original
    }
    #[must_use]
    pub const fn raw_text(&self) -> &'a str {
        self.original.text
    }
    /// Whole source/map coordinates alone do not transfer child authority.
    #[must_use]
    pub const fn source(&self) -> EmbedSource<'a> {
        self.source
    }
    #[must_use]
    pub const fn span(&self) -> Span {
        self.origin.span
    }
    /// Short join at the same original root or element child, without reparse.
    pub fn admitted_for<'s>(
        &'s self,
        selected: &'s NativeTemplateComponent<'a>,
        child: NativeChild<'s, 'a>,
    ) -> Option<NativeTextValueView<'s, 'a>> {
        self.origin
            .matches(selected, &child, &self.original)
            .then_some(NativeTextValueView {
                owner: self,
                selected,
                child,
            })
    }
    /// Moving out ordinary source deliberately drops original-child authority.
    #[must_use]
    pub fn into_source(self) -> EmbedSource<'a> {
        self.source
    }
}

/// Same original child/value join; no whitespace, File or target capability.
pub struct NativeTextValueView<'s, 'a> {
    owner: &'s NativeTextValue<'a>,
    selected: &'s NativeTemplateComponent<'a>,
    child: NativeChild<'s, 'a>,
}
impl<'s, 'a> NativeTextValueView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn child(&self) -> &NativeChild<'s, 'a> {
        &self.child
    }
    #[must_use]
    pub fn observation(&self) -> &'s NativeTextValue<'a> {
        self.owner
    }
}

#[cfg(test)]
mod tests;
