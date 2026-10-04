//! Borrow sealed annotation rows and copy directly into the existing writer.

use super::{SetupEmitError, SetupEmitErrorKind, SetupInput};
use crate::write::{LinkSink, Writer};
use vize_l0::{SourceBlock, Span};

pub(super) fn validate<'owner, 'arena: 'owner>(
    setup: &impl SetupInput<'owner, 'arena>,
) -> Result<(), SetupEmitError> {
    validate_spans(
        setup.source(),
        setup.type_annotations().map(|row| row.span()),
    )
}

fn validate_spans(
    source: SourceBlock<'_>,
    spans: impl Iterator<Item = Span>,
) -> Result<(), SetupEmitError> {
    let mut previous = source.start();
    for span in spans {
        if span.start < previous || span.start >= span.end || !source.contains_block_span(span) {
            return Err(invalid(span));
        }
        previous = span.end;
    }
    Ok(())
}

// The same immutable sealed projection was validated before any writer append.
// SourceBlock checks full-file UTF-8 boundaries; source and row identities cannot
// change between the two borrowed projections. No buffer or AST walk is created.
pub(super) fn append<'owner, 'arena: 'owner, L: LinkSink>(
    setup: &impl SetupInput<'owner, 'arena>,
    writer: &mut Writer<L>,
) -> Result<(), SetupEmitError> {
    let source = setup.source();
    let mut cursor = source.start();
    for annotation in setup.type_annotations() {
        let span = annotation.span();
        if cursor != span.start {
            let text = source
                .source()
                .get((cursor - source.start()) as usize..(span.start - source.start()) as usize)
                .ok_or_else(|| invalid(span))?;
            writer.push_linked(text, Span::new(cursor, span.start));
        }
        cursor = span.end;
    }
    let tail = source
        .source()
        .get((cursor - source.start()) as usize..)
        .ok_or_else(|| invalid(source.span()))?;
    writer.push_linked(tail, Span::new(cursor, source.end()));
    Ok(())
}

fn invalid(span: Span) -> SetupEmitError {
    SetupEmitError {
        span,
        kind: SetupEmitErrorKind::InvalidTypeAnnotation,
    }
}

#[cfg(test)]
mod tests;
