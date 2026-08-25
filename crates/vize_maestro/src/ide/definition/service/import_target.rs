//! Go-to-definition through an imported name (#3893).
//!
//! The checker answers definition at an import site with the local alias
//! declaration — the import statement itself — so the response never left the
//! file, for any import shape. Editors expect the jump to unwrap the alias
//! and land on the exported declaration, the way tsserver does.
//!
//! This follows the import by hand: resolve the specifier (relative, bare
//! package, or tsconfig `paths` alias), then locate the named binding in the
//! target — following `export … from` barrels a bounded number of hops.

use std::fs;
use std::path::Path;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

use crate::ide::IdeContext;
use crate::ide::definition::{helpers, import_resolver::resolve_import_specifier, script};

#[cfg(any(test, feature = "native"))]
mod alias;

#[cfg(any(test, feature = "native"))]
pub(super) use alias::normalize_bound_name_definition;

/// Barrels can chain; three hops covers a package barrel re-exporting a
/// directory barrel re-exporting the module, without risking a cycle walk.
const MAX_REEXPORT_HOPS: usize = 3;

#[cfg(any(test, feature = "native"))]
pub(super) fn definition(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
    let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;
    let (specifier, exported) = importing_specifier(&ctx.content, ctx.offset, &word)?;
    let target = resolve_import_specifier(ctx.uri, &specifier)?;
    locate_export(ctx, &target, &exported, MAX_REEXPORT_HOPS).map(GotoDefinitionResponse::Scalar)
}

/// Definition on an imported component tag, following tsconfig `paths`
/// aliases and re-export barrels (#3932). This must run before the direct-file
/// finder: `import { Primitive } from '@/Primitive'` can resolve first to
/// `src/Primitive/index.ts`, but the editor jump belongs at the source that
/// the barrel re-exports. Uses the same bounded walk as the imported-name jump.
pub(super) fn component_tag_definition(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
    let tag_name = helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
    if !crate::ide::is_component_tag(&tag_name) {
        return None;
    }
    let candidates = crate::ide::component_name_candidates(&tag_name);
    for name in candidates.iter().map(String::as_str) {
        // `import { Widget as LocalWidget }` declares `Widget` in the target,
        // so the lookup key is the exported name, not the local alias.
        let exported = bound_import(&ctx.content, name)
            .map_or_else(|| name.to_owned(), |(_, exported)| exported);
        if let Some(specifier) = helpers::find_import_path(ctx, name)
            && let Some(target) = resolve_import_specifier(ctx.uri, &specifier)
            && let Some(location) = locate_export(ctx, &target, &exported, MAX_REEXPORT_HOPS)
        {
            return Some(GotoDefinitionResponse::Scalar(location));
        }
    }
    None
}

/// The module specifier and exported name of the import statement that both
/// contains `offset` and binds `word`. `None` when the cursor is not on an
/// imported name.
#[cfg(any(test, feature = "native"))]
fn importing_specifier(content: &str, offset: usize, word: &str) -> Option<(String, String)> {
    let (_, _, clause, specifier) = import_statements(content)
        .into_iter()
        .find(|(start, end, _, _)| offset >= *start && offset <= *end)?;
    let exported = bound_source_name(clause, word)?;
    Some((specifier.to_owned(), exported))
}

/// The specifier and exported name for local binding `word`, scanning every
/// import statement — template and component-tag paths have no cursor offset
/// on the import statement to anchor on.
fn bound_import(content: &str, word: &str) -> Option<(String, String)> {
    import_statements(content)
        .into_iter()
        .find_map(|(_, _, clause, specifier)| {
            bound_source_name(clause, word).map(|exported| (specifier.to_owned(), exported))
        })
}

/// The import statements in `content` as `(statement_start, statement_end,
/// clause, specifier)`, where the clause is everything before the specifier's
/// opening quote.
fn import_statements(content: &str) -> Vec<(usize, usize, &str, &str)> {
    let mut statements = Vec::new();
    let mut search_start = 0;
    while let Some(position) = content[search_start..].find("import") {
        let statement_start = search_start + position;
        search_start = statement_start + "import".len();
        let rest = &content[statement_start..];
        let Some((quote_start, quote_end)) = find_specifier_span(rest) else {
            continue;
        };
        statements.push((
            statement_start,
            statement_start + quote_end,
            &rest[..quote_start],
            &rest[quote_start + 1..quote_end],
        ));
    }
    statements
}

/// `(opening_quote_index, closing_quote_index)` of the statement's specifier,
/// relative to the statement start.
fn find_specifier_span(rest: &str) -> Option<(usize, usize)> {
    let line_end = rest.find('\n').map_or(rest.len(), |index| {
        // Multi-line named-import lists keep scanning to the closing quote.
        if rest[..index].contains('{') && !rest[..index].contains('}') {
            rest.len()
        } else {
            index
        }
    });
    let segment = &rest[..line_end];
    let quote_start = segment
        .find('"')
        .into_iter()
        .chain(segment.find('\''))
        .min()?;
    let quote = segment.as_bytes()[quote_start] as char;
    let quote_end = segment[quote_start + 1..].find(quote)? + quote_start + 1;
    Some((quote_start, quote_end))
}

/// The name the target module exports for local binding `word` — the left
/// side of an `as` rename in a named-import clause. `None` when the clause
/// does not bind `word`. Default and namespace imports have no exported name
/// to follow, so the local binding stays the lookup key.
fn bound_source_name(clause: &str, word: &str) -> Option<String> {
    if let (Some(open), Some(close)) = (clause.find('{'), clause.find('}')) {
        for part in clause[open + 1..close].split(',') {
            let part = part.trim().trim_start_matches("type ").trim();
            let (source, bound) = split_rename(part);
            if bound == word {
                return Some(source.to_owned());
            }
        }
    }
    let head = clause.split('{').next().unwrap_or(clause);
    head.split(',')
        .any(|part| {
            let part = part
                .trim()
                .trim_start_matches("import")
                .trim()
                .trim_end_matches("from")
                .trim();
            part == word || part.strip_prefix("* as ").map(str::trim) == Some(word)
        })
        .then(|| word.to_owned())
}

/// `(source, bound)` for a specifier-list entry: `Widget as LocalWidget`
/// splits, a plain `Widget` names both sides.
fn split_rename(part: &str) -> (&str, &str) {
    match part.split_once(" as ") {
        Some((source, bound)) => (source.trim(), bound.trim()),
        None => (part, part),
    }
}

/// The declaration of `word` inside `target`, following re-export barrels.
fn locate_export(ctx: &IdeContext<'_>, target: &Path, word: &str, hops: usize) -> Option<Location> {
    let uri = Url::from_file_path(target).ok()?;
    if target
        .extension()
        .is_some_and(|extension| extension == "vue")
    {
        // A `.vue` module's meaningful position is the file itself, matching
        // how component-tag definition resolves today. Deleted targets are
        // invalid, while an editor-open unsaved target remains navigable.
        if !target.is_file() && !ctx.state.documents.contains(&uri) {
            return None;
        }
        return Some(Location {
            uri,
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
        });
    }
    let content = ctx
        .state
        .documents
        .text(&uri)
        .or_else(|| fs::read_to_string(target).ok())?;

    // A barrel both names the word and points elsewhere; the re-export hop
    // comes first so the jump lands on the real declaration, not the alias.
    if hops > 0
        && let Some((specifier, exported)) = reexport_specifier(&content, word)
        && let Some(next) = resolve_import_specifier(&uri, &specifier)
        && let Some(location) = locate_export(ctx, &next, &exported, hops - 1)
    {
        return Some(location);
    }

    if let Some(binding) = script::find_binding_location_raw(&content, word) {
        let (line, character) = helpers::offset_to_position(&content, binding.offset);
        let position = Position::new(line, character);
        // The end must be a UTF-16 position too: `word.len()` is bytes, so a
        // non-ASCII identifier would overshoot the column.
        let (end_line, end_character) =
            helpers::offset_to_position(&content, binding.offset + word.len());
        let end = Position::new(end_line, end_character);
        return Some(Location {
            uri,
            range: Range::new(position, end),
        });
    }

    None
}

/// The source specifier of an `export … from` clause covering `word`, with the
/// name the next module exports it under — `export { Widget as LocalWidget }`
/// must continue the walk looking for `Widget`.
fn reexport_specifier(content: &str, word: &str) -> Option<(String, String)> {
    let mut search_start = 0;
    while let Some(position) = content[search_start..].find("export") {
        let statement_start = search_start + position;
        search_start = statement_start + "export".len();
        let rest = &content[statement_start..];
        let Some((quote_start, quote_end)) = find_specifier_span(rest) else {
            continue;
        };
        let clause = &rest[..quote_start];
        if !clause.contains("from") {
            continue;
        }
        let source = if let (Some(open), Some(close)) = (clause.find('{'), clause.find('}')) {
            clause[open + 1..close].split(',').find_map(|part| {
                let part = part.trim().trim_start_matches("type ").trim();
                let (source, exported) = split_rename(part);
                (exported == word).then(|| source.to_owned())
            })
        } else if clause.contains('*') {
            Some(word.to_owned())
        } else {
            None
        };
        if let Some(source) = source {
            // `default` is not a locatable declaration name, so keep the
            // requested name for that hop — `export { default as UiButton }`
            // still lands on the module the way it did before.
            let exported = if source == "default" {
                word.to_owned()
            } else {
                source
            };
            return Some((rest[quote_start + 1..quote_end].to_owned(), exported));
        }
    }
    None
}

#[cfg(test)]
mod tests;
