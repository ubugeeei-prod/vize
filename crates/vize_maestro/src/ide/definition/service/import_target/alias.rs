//! Conditional unwrapping for checker answers that stop at an import alias.

use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

use super::{
    MAX_REEXPORT_HOPS, bound_import, bound_source_name, import_statements, locate_export,
    resolve_import_specifier,
};
use crate::ide::IdeContext;
use crate::ide::definition::helpers;

/// Follow an imported binding only when the checker answered with that
/// binding's authored import alias. This preserves lexical shadowing in
/// template expressions while avoiding an import self-jump.
pub(in crate::ide::definition::service) fn unwrap_bound_name_definition(
    ctx: &IdeContext<'_>,
    response: &GotoDefinitionResponse,
) -> Option<GotoDefinitionResponse> {
    let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;
    let word_start = word_start_at_offset(&ctx.content, ctx.offset)?;
    if word_start
        .checked_sub(1)
        .and_then(|index| ctx.content.as_bytes().get(index))
        == Some(&b'.')
    {
        return None;
    }
    let location = match response {
        GotoDefinitionResponse::Scalar(location) => location,
        GotoDefinitionResponse::Array(locations) if locations.len() == 1 => &locations[0],
        GotoDefinitionResponse::Array(_) | GotoDefinitionResponse::Link(_) => return None,
    };
    if !same_document_uri(&location.uri, ctx.uri) {
        return None;
    }
    let definition_offset = crate::ide::position_to_offset(
        &ctx.content,
        location.range.start.line,
        location.range.start.character,
    )?;
    let (specifier, exported) = bound_import(&ctx.content, &word)?;
    let points_to_import_alias =
        import_statements(&ctx.content)
            .into_iter()
            .any(|(start, end, clause, _)| {
                definition_offset >= start
                    && definition_offset <= end
                    && bound_source_name(clause, &word).is_some()
            });
    if !points_to_import_alias {
        return None;
    }
    let target = resolve_import_specifier(ctx.uri, &specifier)?;
    locate_export(ctx, &target, &exported, MAX_REEXPORT_HOPS).map(GotoDefinitionResponse::Scalar)
}

fn word_start_at_offset(content: &str, offset: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut start = offset.min(bytes.len());
    while start > 0 && helpers::is_word_char(bytes[start - 1]) {
        start -= 1;
    }
    let has_word = bytes
        .get(start)
        .is_some_and(|byte| helpers::is_word_char(*byte));
    has_word.then_some(start)
}

fn same_document_uri(left: &Url, right: &Url) -> bool {
    if left == right {
        return true;
    }
    let (Ok(left), Ok(right)) = (left.to_file_path(), right.to_file_path()) else {
        return false;
    };
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}
