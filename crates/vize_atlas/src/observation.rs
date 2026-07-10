//! Provider-attributed diagnostics, fallbacks, and notes.

use crate::{ProductRequest, ProviderId, Shared, SourceId, SourceRange};

/// Domain-neutral category for a provider-side observation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum ObservationKind {
    /// A source or product diagnostic.
    Diagnostic,
    /// The provider selected a compatibility or recovery path.
    Fallback,
    /// Informational execution metadata useful to tools or inspectors.
    Note,
}

/// Structured side outcome tied to the exact request and provider that emitted it.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProviderObservation {
    request: ProductRequest,
    provider: ProviderId,
    source: SourceId,
    range: Option<SourceRange>,
    kind: ObservationKind,
    code: Shared<str>,
    message: Shared<str>,
}

impl ProviderObservation {
    pub(crate) fn new(
        request: ProductRequest,
        provider: ProviderId,
        source: SourceId,
        range: Option<SourceRange>,
        kind: ObservationKind,
        code: impl Into<Shared<str>>,
        message: impl Into<Shared<str>>,
    ) -> Self {
        Self {
            request,
            provider,
            source,
            range,
            kind,
            code: code.into(),
            message: message.into(),
        }
    }

    /// Product request whose provider emitted the observation.
    pub const fn request(&self) -> ProductRequest {
        self.request
    }

    /// Concrete provider that emitted the observation.
    pub const fn provider(&self) -> ProviderId {
        self.provider
    }

    /// Source to which the observation applies.
    pub const fn source(&self) -> SourceId {
        self.source
    }

    /// Optional source byte range.
    pub const fn range(&self) -> Option<SourceRange> {
        self.range
    }

    pub const fn kind(&self) -> ObservationKind {
        self.kind
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
