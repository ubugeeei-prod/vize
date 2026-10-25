//! The image-alt decision and diagnostic text, independent of input syntax.

use std::borrow::Cow;
use vize_l0::{
    Span,
    diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage},
};

#[derive(Clone, Copy)]
pub(crate) struct ImageAltPolicy {
    image: bool,
}

impl ImageAltPolicy {
    #[inline]
    pub(crate) fn for_tag(tag: &str) -> Self {
        Self {
            image: tag.eq_ignore_ascii_case("img"),
        }
    }

    #[inline]
    pub(crate) fn applies(self) -> bool {
        self.image
    }

    #[inline]
    pub(crate) fn is_missing(self, has_alt: bool) -> bool {
        self.image && !has_alt
    }
}

/// An immutable catalog selection, captured only when a diagnostic is needed.
pub(crate) struct ImageAltMessages {
    message: Cow<'static, str>,
    help: Cow<'static, str>,
}

impl ImageAltMessages {
    pub(crate) fn new(messages: &impl MessageLookup) -> Self {
        Self::from_lookup(|key| messages.lookup(key))
    }

    pub(crate) fn from_lookup(mut lookup: impl FnMut(&'static str) -> Cow<'static, str>) -> Self {
        Self {
            message: lookup("a11y/img-alt.message"),
            help: lookup("a11y/img-alt.help"),
        }
    }

    pub(crate) fn message(&self) -> &str {
        self.message.as_ref()
    }

    pub(crate) fn help(&self) -> &str {
        self.help.as_ref()
    }

    pub(crate) fn diagnostic(&self, range: Span) -> Diagnostic {
        Diagnostic::new(Advisory::Warning, Stage::Surface, range, self.message())
            .with_part(DiagnosticPart::new(PartKind::Help, range, self.help()))
    }
}
