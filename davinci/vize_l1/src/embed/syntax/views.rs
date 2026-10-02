use oxc_ast::ast::{Comment, CommentKind};
use oxc_diagnostics::{LabeledSpan, OxcCode, OxcDiagnostic, Severity};
use vize_l0::Span;

use super::{SourceError, coordinates::Coordinates};

/// An OXC comment retained by the same parse, with no generated coordinates.
#[derive(Clone, Copy)]
pub struct CommentView<'d, 'a> {
    pub(super) comment: &'d Comment,
    pub(super) coordinates: Coordinates<'a>,
}

impl core::fmt::Debug for CommentView<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CommentView")
            .field("kind", &self.kind())
            .field("decoded", &self.decoded_span())
            .field("authored", &self.authored_span())
            .finish()
    }
}

impl<'a> CommentView<'_, 'a> {
    #[must_use]
    pub const fn kind(self) -> CommentKind {
        self.comment.kind
    }

    pub fn decoded_span(self) -> Result<Span, SourceError> {
        self.coordinates.decoded_span(self.comment.span)
    }

    pub fn authored_span(self) -> Result<Span, SourceError> {
        self.coordinates.source.authored_span(self.decoded_span()?)
    }

    /// Full decoded comment bytes, including delimiters.
    pub fn text(self) -> Result<&'a str, SourceError> {
        let span = self.decoded_span()?;
        self.coordinates
            .source
            .text()
            .get(span.start as usize..span.end as usize)
            .ok_or(SourceError::InvalidDecodedSpan)
    }
}

/// Diagnostic metadata is retained in OXC's original owned record. Only its
/// label locations are exposed through conservative decoded/authored views.
#[derive(Clone, Copy)]
pub struct DiagnosticView<'d, 'a> {
    pub(super) diagnostic: &'d OxcDiagnostic,
    pub(super) coordinates: Coordinates<'a>,
}

impl core::fmt::Debug for DiagnosticView<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DiagnosticView")
            .field("message", &self.message())
            .field("severity", &self.severity())
            .field("label_count", &self.diagnostic.labels.len())
            .finish()
    }
}

impl<'d, 'a> DiagnosticView<'d, 'a> {
    #[must_use]
    pub fn message(self) -> &'d str {
        &self.diagnostic.message
    }
    #[must_use]
    pub fn severity(self) -> Severity {
        self.diagnostic.severity
    }
    #[must_use]
    pub fn code(self) -> &'d OxcCode {
        &self.diagnostic.code
    }
    #[must_use]
    pub fn help(self) -> Option<&'d str> {
        self.diagnostic.help.as_deref()
    }
    #[must_use]
    pub fn note(self) -> Option<&'d str> {
        self.diagnostic.note.as_deref()
    }
    #[must_use]
    pub fn url(self) -> Option<&'d str> {
        self.diagnostic.url.as_deref()
    }

    pub fn labels(self) -> impl Iterator<Item = DiagnosticLabel<'d, 'a>> {
        self.diagnostic
            .labels
            .iter()
            .map(move |label| DiagnosticLabel {
                label,
                coordinates: self.coordinates,
            })
    }
}

#[derive(Clone, Copy)]
pub struct DiagnosticLabel<'d, 'a> {
    label: &'d LabeledSpan,
    coordinates: Coordinates<'a>,
}

impl core::fmt::Debug for DiagnosticLabel<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DiagnosticLabel")
            .field("message", &self.message())
            .field("decoded", &self.decoded_span())
            .field("authored", &self.authored_span())
            .finish()
    }
}

impl<'d> DiagnosticLabel<'d, '_> {
    #[must_use]
    pub fn message(self) -> Option<&'d str> {
        self.label.label()
    }
    #[must_use]
    pub fn primary(self) -> bool {
        self.label.primary()
    }

    pub fn decoded_span(self) -> Result<Span, SourceError> {
        let start = self.label.offset();
        let end = start
            .checked_add(self.label.len())
            .ok_or(SourceError::SourceTooLarge)?;
        self.coordinates.diagnostic_span(start, end)
    }

    pub fn authored_span(self) -> Result<Span, SourceError> {
        self.coordinates
            .source
            .authored_covering_span(self.decoded_span()?)
    }
}
