//! Acceptance of an output-target answer.

use super::{
    AcceptedEmit, EMIT_DOCUMENT_PAGE_SCHEMA, EmitDocument, EmitError, EmitRequest, Emitted,
    S3_PAGE_SCHEMA,
};
use crate::accept::read_page;
use crate::contract::{S2_PAGE_SCHEMA, Span};

/// Accept a guest's emission of `request`.
///
/// # Errors
///
/// The first check the answer fails, as an [`EmitError`].
pub fn accept_emitted(request: &EmitRequest, emitted: Emitted) -> Result<AcceptedEmit, EmitError> {
    schema("s2-page", S2_PAGE_SCHEMA, request.s2.schema_version)?;
    schema("s3-page", S3_PAGE_SCHEMA, request.s3.schema_version)?;
    let document = read_page(
        "emit-document-page",
        EMIT_DOCUMENT_PAGE_SCHEMA,
        &emitted.document,
    )
    .map_err(EmitError::Page)?;
    check_generated(&document)?;
    for (index, diagnostic) in emitted.diagnostics.iter().enumerate() {
        let spans =
            core::iter::once(diagnostic.span).chain(diagnostic.parts.iter().map(|p| p.span));
        for span in spans {
            if !inside_authored(&document, span) {
                return Err(EmitError::DiagnosticSpan { index, span });
            }
        }
    }
    Ok(AcceptedEmit { emitted, document })
}

fn schema(page: &'static str, reads: u32, found: u32) -> Result<(), EmitError> {
    if found == reads {
        Ok(())
    } else {
        Err(EmitError::RequestSchema { page, found, reads })
    }
}

fn check_generated(document: &EmitDocument) -> Result<(), EmitError> {
    let text = document.text.as_str();
    for (index, link) in document.links.iter().enumerate() {
        let span = link.generated;
        let start = span.start as usize;
        let end = span.end as usize;
        let inside =
            end <= text.len() && text.is_char_boundary(start) && text.is_char_boundary(end);
        if !inside {
            return Err(EmitError::GeneratedRange { index, span });
        }
    }
    Ok(())
}

fn inside_authored(document: &EmitDocument, span: Span) -> bool {
    span.start <= span.end
        && document
            .links
            .iter()
            .any(|link| link.authored.start <= span.start && span.end <= link.authored.end)
}
