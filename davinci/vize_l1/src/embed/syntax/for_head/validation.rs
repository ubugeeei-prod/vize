use super::{EmbedHole, ForHeadHole, ForHeadPart, NativeForHead, RetainedSlotParams, SourceError};
use oxc_ast::ast::FormalParameter;

pub(super) fn refused(head: &NativeForHead<'_>) -> Option<ForHeadHole> {
    let aliases = match head.aliases() {
        Some(Ok(aliases)) => aliases,
        _ => return Some(ForHeadHole::RejectedPart(ForHeadPart::Aliases)),
    };
    let collection = match head.collection() {
        Some(Ok(collection)) => collection,
        _ => return Some(ForHeadHole::RejectedPart(ForHeadPart::Collection)),
    };
    if let Some(hole) = aliases.hole() {
        return Some(ForHeadHole::AliasesUnavailable(hole));
    }
    if let Some(hole) = collection.hole() {
        return Some(ForHeadHole::CollectionUnavailable(hole));
    }
    let Some(parameters) = aliases.parameters() else {
        return Some(ForHeadHole::AliasesUnavailable(
            EmbedHole::InvalidWrappedShape,
        ));
    };
    if collection.expression().is_none() {
        return Some(ForHeadHole::CollectionUnavailable(
            EmbedHole::InvalidExpressionShape,
        ));
    }
    if aliases.rest().is_some() {
        return Some(ForHeadHole::RestAlias);
    }
    if parameters.is_empty() {
        return Some(ForHeadHole::EmptyAliases);
    }
    if parameters.len() > 3 {
        return Some(ForHeadHole::ExtraAliases(parameters.len()));
    }
    let Some(last) = parameters.last() else {
        return Some(ForHeadHole::EmptyAliases);
    };
    match alias_tail_is_trivia(aliases, last) {
        Ok(true) => None,
        Ok(false) => Some(ForHeadHole::TrailingAliasSyntax),
        Err(error) => Some(ForHeadHole::SourceBoundary(error)),
    }
}

/// A trailing comma authors a sparse position even though Params accepts it.
/// Inspect only the tail after the final true root, skipping exact OXC comments;
/// this neither scans alias commas nor invents positions inside lexical forms.
fn alias_tail_is_trivia(
    aliases: &RetainedSlotParams<'_>,
    last: &FormalParameter<'_>,
) -> Result<bool, SourceError> {
    let text = aliases.source().text();
    let mut cursor = aliases.decoded_span(last.span)?.end as usize;
    for comment in aliases.comments() {
        let span = comment.decoded_span()?;
        if span.end as usize <= cursor {
            continue;
        }
        if (span.start as usize) < cursor {
            return Err(SourceError::InvalidDecodedSpan);
        }
        let before = text
            .get(cursor..span.start as usize)
            .ok_or(SourceError::InvalidDecodedSpan)?;
        if !before.trim().is_empty() {
            return Ok(false);
        }
        cursor = span.end as usize;
    }
    Ok(text
        .get(cursor..)
        .ok_or(SourceError::InvalidDecodedSpan)?
        .trim()
        .is_empty())
}
