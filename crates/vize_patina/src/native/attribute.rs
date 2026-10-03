//! Checked readonly header projections; expression and value bytes stay opaque.

use vize_l0::{SourceBlock, Span};
use vize_l1::{Attribute, Token};

use super::NativeLintRefusal;

pub(super) fn span(block: SourceBlock<'_>, text: &str) -> Result<Span, NativeLintRefusal> {
    block
        .span_of(text)
        .filter(|span| block.contains_block_span(*span))
        .ok_or(NativeLintRefusal::SourceMismatch)
}

pub(super) fn token(block: SourceBlock<'_>, token: &Token<'_>) -> Result<(), NativeLintRefusal> {
    if token.is_missing() {
        return Err(NativeLintRefusal::Hole);
    }
    for text in [token.leading, token.text] {
        if !text.is_empty() {
            span(block, text)?;
        }
    }
    Ok(())
}

pub(super) fn attribute(
    block: SourceBlock<'_>,
    attribute: &Attribute<'_>,
) -> Result<Span, NativeLintRefusal> {
    token(block, &attribute.name)?;
    if attribute.name.text.is_empty() {
        return Err(NativeLintRefusal::Hole);
    }
    if attribute.eq.is_some() != attribute.value.is_some() {
        return Err(NativeLintRefusal::Hole);
    }
    if let Some(eq) = &attribute.eq {
        token(block, eq)?;
    }
    if let Some(value) = &attribute.value {
        if value.open_quote.is_some() != value.close_quote.is_some() {
            return Err(NativeLintRefusal::Hole);
        }
        if let Some(open) = &value.open_quote {
            token(block, open)?;
        }
        token(block, &value.content)?;
        if let Some(close) = &value.close_quote {
            token(block, close)?;
        }
    }
    span(block, attribute.name.text)
}

pub(super) fn project<'a>(
    block: SourceBlock<'a>,
    span: Span,
) -> Result<&'a str, NativeLintRefusal> {
    if !block.contains_block_span(span) {
        return Err(NativeLintRefusal::SourceMismatch);
    }
    block
        .root_source()
        .get(span.start as usize..span.end as usize)
        .ok_or(NativeLintRefusal::SourceMismatch)
}
