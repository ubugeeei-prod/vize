//! Existing whole-source local export lookup, with the same transaction.

use super::{ReferenceSink, ReferenceSource, ResolutionError, Resolver, Usage};

pub(in crate::resolution) fn export_local<'a>(
    source: ReferenceSource<'a>,
    span: oxc_span::Span,
    name: &'a str,
    sink: &mut impl ReferenceSink<'a>,
) -> Result<(), ResolutionError> {
    let checkpoint = sink.checkpoint();
    let mut resolver = Resolver::new(source, sink);
    let result = resolver.reference(span, name, Usage::Read, false);
    if result.is_err() {
        resolver.sink.rollback(checkpoint);
    }
    result
}
