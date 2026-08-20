//! Conditional unwrapping for checker answers that stop at an import alias.

use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

use super::{
    MAX_REEXPORT_HOPS, bound_import, bound_source_name, import_statements, locate_export,
    resolve_import_specifier,
};
use crate::ide::IdeContext;
use crate::ide::definition::helpers;

/// Normalize a checker answer that may stop at an authored import alias.
///
/// Non-alias answers are preserved exactly, including template-local
/// shadowing. Once the answer is known to be an import alias, however, an
/// unresolvable target is not a valid definition and must become `None`
/// instead of falling back to the stale alias.
pub(in crate::ide::definition::service) fn normalize_bound_name_definition(
    ctx: &IdeContext<'_>,
    response: GotoDefinitionResponse,
) -> Option<GotoDefinitionResponse> {
    let Some(word) = helpers::get_word_at_offset(&ctx.content, ctx.offset) else {
        return Some(response);
    };
    let Some(word_start) = word_start_at_offset(&ctx.content, ctx.offset) else {
        return Some(response);
    };
    if word_start
        .checked_sub(1)
        .and_then(|index| ctx.content.as_bytes().get(index))
        == Some(&b'.')
    {
        return Some(response);
    }
    let location = match &response {
        GotoDefinitionResponse::Scalar(location) => location,
        GotoDefinitionResponse::Array(locations) if locations.len() == 1 => &locations[0],
        GotoDefinitionResponse::Array(_) | GotoDefinitionResponse::Link(_) => {
            return Some(response);
        }
    };
    if !same_document_uri(&location.uri, ctx.uri) {
        return Some(response);
    }
    let Some(definition_offset) = crate::ide::position_to_offset(
        &ctx.content,
        location.range.start.line,
        location.range.start.character,
    ) else {
        return Some(response);
    };
    let Some((specifier, exported)) = bound_import(&ctx.content, &word) else {
        return Some(response);
    };
    let points_to_import_alias =
        import_statements(&ctx.content)
            .into_iter()
            .any(|(start, end, clause, _)| {
                definition_offset >= start
                    && definition_offset <= end
                    && bound_source_name(clause, &word).is_some()
            });
    if !points_to_import_alias {
        return Some(response);
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

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

    use super::normalize_bound_name_definition;
    use crate::ide::IdeContext;
    use crate::server::ServerState;

    fn scalar_at(uri: &Url, content: &str, offset: usize, len: usize) -> GotoDefinitionResponse {
        let (line, character) = crate::ide::offset_to_position(content, offset);
        GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range: Range::new(
                Position::new(line, character),
                Position::new(line, character + len as u32),
            ),
        })
    }

    #[test]
    fn a_deleted_import_alias_normalizes_to_no_definition() {
        let workspace = tempfile::tempdir().unwrap();
        let parent_path = workspace.path().join("Parent.vue");
        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>
<template><Child /></template>
"#;
        let uri = Url::from_file_path(parent_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
        state.update_virtual_docs(&uri, source);
        let ctx = IdeContext::new(&state, &uri, source.rfind("Child").unwrap()).unwrap();
        let alias = scalar_at(&uri, source, source.find("Child").unwrap(), "Child".len());

        assert!(normalize_bound_name_definition(&ctx, alias).is_none());
    }

    #[test]
    fn a_non_alias_definition_is_preserved_exactly() {
        let workspace = tempfile::tempdir().unwrap();
        let parent_path = workspace.path().join("Parent.vue");
        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
const children = []
</script>
<template><div v-for="Child in children">{{ Child }}</div></template>
"#;
        let uri = Url::from_file_path(parent_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
        state.update_virtual_docs(&uri, source);
        let ctx = IdeContext::new(&state, &uri, source.rfind("Child").unwrap()).unwrap();
        let response = scalar_at(
            &uri,
            source,
            source.find("v-for=\"Child").unwrap() + "v-for=\"".len(),
            "Child".len(),
        );

        assert_eq!(
            normalize_bound_name_definition(&ctx, response.clone()),
            Some(response)
        );
    }

    #[test]
    fn an_open_unsaved_import_target_remains_navigable() {
        let workspace = tempfile::tempdir().unwrap();
        let parent_path = workspace.path().join("Parent.vue");
        let child_path = workspace.path().join("Child.vue");
        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>
<template><Child /></template>
"#;
        let uri = Url::from_file_path(parent_path).unwrap();
        let child_uri = Url::from_file_path(child_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
        state.documents.open(
            child_uri.clone(),
            "<template />\n".to_owned(),
            1,
            "vue".to_owned(),
        );
        state.update_virtual_docs(&uri, source);
        let ctx = IdeContext::new(&state, &uri, source.rfind("Child").unwrap()).unwrap();
        let alias = scalar_at(&uri, source, source.find("Child").unwrap(), "Child".len());

        assert_eq!(
            normalize_bound_name_definition(&ctx, alias),
            Some(GotoDefinitionResponse::Scalar(Location {
                uri: child_uri,
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            }))
        );
    }
}
