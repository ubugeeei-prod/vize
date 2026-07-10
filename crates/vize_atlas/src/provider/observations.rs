use super::ProviderContext;
use crate::{ObservationKind, ProviderObservation, Shared, SourceId, SourceRange};

impl ProviderContext<'_> {
    /// Record a structured diagnostic, fallback, or note for the current source.
    pub fn observe(
        &mut self,
        kind: ObservationKind,
        code: impl Into<Shared<str>>,
        message: impl Into<Shared<str>>,
        range: Option<SourceRange>,
    ) {
        self.observe_for_source(self.source.id(), kind, code, message, range);
    }

    /// Record an observation that applies to another source in a declared project query.
    pub fn observe_for_source(
        &mut self,
        source: SourceId,
        kind: ObservationKind,
        code: impl Into<Shared<str>>,
        message: impl Into<Shared<str>>,
        range: Option<SourceRange>,
    ) {
        self.observations.push(ProviderObservation::new(
            self.request,
            self.provider,
            source,
            range,
            kind,
            code,
            message,
        ));
    }
}
